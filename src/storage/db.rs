use crate::parry::bounding_volume::Aabb;
use crate::rapier::data::Coarena;
use crate::rapier::dynamics::RigidBodyHandle;
use bevy::prelude::Resource;
use bevy::utils::Uuid;
use bevy_rapier::math::Vect;
use dashmap::{DashMap, DashSet};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::messages::{BodyAssignment, ImpulseJointAssignment};
use steadyum_api_types::objects::{BodyPositionObject, ColdBodyObject, RegionList, WarmBodyObject};
use steadyum_api_types::region_db::PartitionnerServer;
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::ZenohContext;

pub struct NewObjectCommand {
    pub uuid: Uuid,
    pub handle: RigidBodyHandle,
    pub cold_object: ColdBodyObject,
    pub warm_object: WarmBodyObject,
}

pub enum DbCommand {
    NewObjects { objects: Vec<NewObjectCommand> },
    NewJoints { joints: Vec<ImpulseJointAssignment> },
}

#[derive(Clone, Debug, Default)]
pub struct LatestRigidBodyData {
    pub region: usize,
    pub data: BodyPositionObject,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CameraPos {
    pub position: Vect,
    pub dir: Vect,
}

impl CameraPos {
    pub fn visible_regions(&self) -> Vec<SimulationBounds> {
        let camera_a = self.position;
        let camera_b = self.position + self.dir * SimulationBounds::DEFAULT_WIDTH as f32 * 20.0;
        let camera_aabb = Aabb::new(camera_a.min(camera_b).into(), camera_a.max(camera_b).into());

        SimulationBounds::intersecting_aabb(camera_aabb, SimulationBounds::DEFAULT_WIDTH)
    }
}

pub struct NewObjectData {
    pub pos: BodyPositionObject,
    pub body: ColdBodyObject,
}

#[derive(Resource)]
pub struct DbContext {
    pub kvs: KvsContext,
    pub zenoh: Mutex<ZenohContext>,
    pub commands_snd: flume::Sender<DbCommand>,
    pub to_monitor: Arc<DashSet<Uuid>>,
    pub latest_data: Arc<RwLock<Coarena<LatestRigidBodyData>>>,
    pub camera: Arc<RwLock<CameraPos>>,
    pub uuid2rb: Arc<DashMap<Uuid, RigidBodyHandle>>,
    pub rb2uuid: HashMap<RigidBodyHandle, Uuid>,
    pub region_list: Arc<RwLock<RegionList>>,
    pub new_objects_rcv: flume::Receiver<NewObjectData>,
    pub partitionner: Arc<PartitionnerServer>,
    pub is_running: bool,
}

pub fn spawn_db_thread() -> DbContext {
    let (commands_snd, commands_rcv) = flume::unbounded();
    let (new_objects_snd, new_objects_rcv) = flume::unbounded();
    let to_monitor = Arc::new(DashSet::<Uuid>::new());
    let latest_data = Arc::new(RwLock::new(Coarena::new()));
    let camera = Arc::new(RwLock::new(CameraPos::default()));
    let uuid2rb = Arc::new(DashMap::new());
    let region_list = Arc::new(RwLock::new(RegionList::default()));
    let partitionner = Arc::new(PartitionnerServer::new().unwrap());

    {
        let uuid2rb = uuid2rb.clone();
        let partitionner = partitionner.clone();

        std::thread::spawn(move || {
            /*
             * Command loop.
             */
            while let Ok(command) = commands_rcv.recv() {
                match command {
                    DbCommand::NewJoints { .. } => {
                        todo!()
                        // put_json(
                        //     &partitionner,
                        //     &PartitionnerMessage::AssignMulipleImpulseJoints { joints },
                        // )
                        // .unwrap();
                    }
                    DbCommand::NewObjects { objects } => {
                        for objects in objects.chunks(256) {
                            let bodies_to_insert: Vec<_> = objects
                                .iter()
                                .inspect(|obj| {
                                    uuid2rb.insert(obj.uuid, obj.handle);
                                })
                                .map(|obj| BodyAssignment {
                                    uuid: obj.uuid,
                                    cold: obj.cold_object.clone(),
                                    warm: obj.warm_object.clone(),
                                })
                                .collect();
                            dbg!("Sending objects query to the partitionner!");
                            partitionner.insert_objects(bodies_to_insert).unwrap();
                        }
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    {
        let uuid2rb = uuid2rb.clone();
        let latest_data = latest_data.clone();
        let region_list = region_list.clone();
        let partitionner = partitionner.clone();

        // let camera = camera.clone();
        std::thread::spawn(move || {
            /*
             * Init S3
             */
            let mut kvs = KvsContext::new().unwrap();
            let mut num_region_ids = 0;
            let mut region_ids = HashMap::new();
            let mut known_new_objects = HashSet::new();

            /*
             * Position reading loop.
             */
            loop {
                let new_region_list: RegionList = partitionner.list_regions().unwrap_or_default();
                let all_keys = &new_region_list.keys;
                let all_ids = all_keys.iter().map(|key| {
                    *region_ids.entry(key.clone()).or_insert_with(|| {
                        num_region_ids += 1;
                        num_region_ids - 1
                    })
                });

                let mut timestamps = vec![];

                if let Ok(all_data) = kvs.get_multiple_warm(&all_keys) {
                    let mut new_latest_data = Coarena::new();
                    let mut new_unknown_objects = vec![];

                    for ((data, region), reg_key) in
                        all_data.into_iter().zip(all_ids).zip(all_keys.iter())
                    {
                        if let Some(data) = data {
                            timestamps.push((data.timestamp, reg_key.clone(), data.objects.len()));
                            for object in data.objects {
                                let data = LatestRigidBodyData {
                                    region,
                                    data: object,
                                };
                                if let Some(handle) = uuid2rb.get(&object.uuid) {
                                    // println!("New data: {:?}, {:?}", handle, data);
                                    new_latest_data.insert(handle.0, data);
                                } else if known_new_objects.insert(object.uuid) {
                                    // Grab the cold object data.
                                    new_unknown_objects.push(object);
                                }
                            }
                        }
                    }
                    *latest_data.write().unwrap() = new_latest_data;

                    // Get all the unknown objects.
                    if !new_unknown_objects.is_empty() {
                        dbg!(new_unknown_objects.len());
                        let all_unknown_uuids: Vec<_> =
                            new_unknown_objects.iter().map(|obj| obj.uuid).collect();
                        if let Ok(all_cold) = kvs.get_multiple_cold(&all_unknown_uuids) {
                            for (obj, cold) in
                                new_unknown_objects.into_iter().zip(all_cold.into_iter())
                            {
                                if let Some(cold) = cold {
                                    let new_object_data = NewObjectData {
                                        pos: obj,
                                        body: cold,
                                    };
                                    new_objects_snd.send(new_object_data).unwrap();
                                } else {
                                    known_new_objects.remove(&obj.uuid);
                                }
                            }
                        }
                    }
                }

                if !timestamps.is_empty() {
                    timestamps.sort_by_key(|elt| elt.0);
                }

                *region_list.write().unwrap() = new_region_list;
            }
        });
    }

    let kvs = KvsContext::new().unwrap();
    let zenoh = ZenohContext::new().unwrap();
    DbContext {
        commands_snd,
        kvs,
        zenoh: Mutex::new(zenoh),
        to_monitor,
        latest_data,
        uuid2rb,
        rb2uuid: HashMap::new(),
        new_objects_rcv,
        camera,
        region_list,
        partitionner,
        is_running: false,
    }
}
