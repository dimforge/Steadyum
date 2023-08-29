use std::collections::{HashMap, HashSet};
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::messages::{
    ImpulseJointAssignment, PartitionnerMessage, RunnerMessage, PARTITIONNER_QUEUE,
};
use steadyum_api_types::objects::{RegionList, WarmBodyObject, WarmBodyObjectSet};
use steadyum_api_types::rapier::parry::bounding_volume::{Aabb, BoundingVolume};
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::{put_json, ZenohContext};
use uuid::Uuid;
use zenoh::prelude::sync::SyncResolve;
use zenoh::prelude::SplitBuffer;

pub struct Runner {
    pub process: Child,
    pub assigned_objects: HashSet<Uuid>,
    pub key: String,
    pub region: SimulationBounds,
    pub port: u32,
    pub started: bool,
    pub pending: Vec<RunnerMessage>,
}

impl Runner {
    fn send_pending_if_started(&mut self, zenoh: &ZenohContext) {
        if self.started && !self.pending.is_empty() {
            let publisher = zenoh
                .session
                .declare_publisher(&self.key)
                .res_sync()
                .expect("J");

            for msg in self.pending.drain(..) {
                put_json(&publisher, &msg);
            }
        }
    }
}

pub struct LiveRunners {
    pub next_port_id: u32,
    pub runners: HashMap<String, Runner>,
    pub object2runner: HashMap<Uuid, String>,
}

impl Default for LiveRunners {
    fn default() -> Self {
        Self {
            next_port_id: 10_000,
            runners: HashMap::default(),
            object2runner: HashMap::default(),
        }
    }
}

fn assign_object_to_region(
    zenoh: &ZenohContext,
    db: &mut KvsContext,
    runners: &mut LiveRunners,
    region: SimulationBounds,
    uuid: Uuid,
    warm_object: WarmBodyObject,
    curr_step: u64,
    is_running: bool,
) -> anyhow::Result<()> {
    let runner_queue = region.zenoh_queue_key();
    let runner_existed = runners.runners.contains_key(&runner_queue);

    let runner_message = RunnerMessage::ReAssignObject {
        uuid,
        warm_object: warm_object.clone(),
    };

    // Spawn the worker if it doesn’t exist yet.
    let runner = runners
        .runners
        .entry(runner_queue.clone())
        .or_insert_with(|| {
            dbg!("Spawning child process", region.zenoh_queue_key());
            // HACK: clear the runner’s files before creating it.
            // Won’t be needed once we support scene names.
            db.put_warm(
                &region.runner_key(),
                &WarmBodyObjectSet {
                    timestamp: curr_step,
                    objects: vec![],
                },
            )
            .unwrap();

            // Spawn the runner.
            let port = runners.next_port_id;
            runners.next_port_id += 1;
            let process = region
                .command("steadyum-runner.exe", curr_step, port)
                .spawn()
                .unwrap();
            Runner {
                process,
                assigned_objects: HashSet::new(),
                key: region.runner_key(),
                region,
                port,
                started: false,
                pending: vec![],
            }
        });
    runner.assigned_objects.insert(uuid.to_owned());
    runners
        .object2runner
        .insert(uuid.to_owned(), runner_queue.clone());

    if !runner_existed {
        runner.pending.push(RunnerMessage::StartStop {
            running: is_running,
        });
    }

    runner.pending.push(runner_message);
    runner.send_pending_if_started(&zenoh);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.init();

    let zenoh = ZenohContext::new().expect("G");
    let consumer = zenoh
        .session
        .declare_subscriber(PARTITIONNER_QUEUE)
        .res_sync()
        .expect("H");

    // Kvs for the region list.
    let mut db = KvsContext::new()?;

    let mut runners = LiveRunners::default();
    let mut is_running = false;
    let mut step_id = 0;
    println!("Waiting for messages. Press Ctrl-C to exit.");

    db.put("region_list", &RegionList::default());

    let assign_single_joint = |joint: ImpulseJointAssignment, runners: &mut LiveRunners| {
        let runner_message = RunnerMessage::AssignJoint(joint);

        if let (Some(run_key1), Some(run_key2)) = (
            runners.object2runner.get(&joint.body1),
            runners.object2runner.get(&joint.body2),
        ) {
            if let (Some(runner1), Some(runner2)) =
                (runners.runners.get(run_key1), runners.runners.get(run_key2))
            {
                zenoh.put_json(&run_key1, &runner_message);
                if runner1.region != runner2.region {
                    zenoh.put_json(&run_key2, &runner_message);
                }

                // if runner1.region >= runner2.region {
                //     zenoh.publish(&runner_message, &run_key1);
                // } else {
                //     zenoh.publish(&runner_message, &run_key2);
                // }
            }
        }
    };

    let assign_single_object = |uuid: Uuid,
                                origin,
                                aabb: Aabb,
                                warm_object,
                                dynamic,
                                runners: &mut LiveRunners,
                                db: &mut KvsContext,
                                curr_step: u64,
                                is_running| {
        let regions = if dynamic {
            vec![SimulationBounds::from_point(
                aabb.maxs,
                SimulationBounds::DEFAULT_WIDTH,
            )]
        } else {
            SimulationBounds::intersecting_aabb(aabb.loosened(1.0), SimulationBounds::DEFAULT_WIDTH)
        };

        if let Some(origin) = runners.object2runner.remove(&uuid) {
            if let Some(runner) = runners.runners.get_mut(&origin) {
                // Remove previous assignment.
                runner.assigned_objects.remove(&uuid);
            }
        }

        for region in regions {
            assign_object_to_region(
                &zenoh,
                db,
                runners,
                region,
                uuid,
                warm_object,
                curr_step,
                is_running,
            )?;
        }

        Ok::<(), anyhow::Error>(())
    };

    const STEP_INCREMENTS: u32 = 20;
    let mut pending_acks = 0;
    let mut curr_step = 0;
    let mut last_ack_time = Instant::now();

    loop {
        let sample = consumer.recv().expect("I");
        let num_regions_before = runners.runners.len();
        let payload = sample.value.payload.contiguous();
        let body = String::from_utf8_lossy(&payload);

        if let Ok(message) = serde_json::from_str::<PartitionnerMessage>(&body) {
            match message {
                PartitionnerMessage::AckSteps { stopped, origin } => {
                    pending_acks -= 1;

                    if stopped {
                        runners.runners.remove(&origin);
                    }
                }
                PartitionnerMessage::StartStop { running } => {
                    if running != is_running {
                        let message = RunnerMessage::StartStop { running };
                        is_running = running;

                        for (_, runner) in &mut runners.runners {
                            runner.pending.push(message.clone());
                            runner.send_pending_if_started(&zenoh);
                        }
                    }
                }
                PartitionnerMessage::AssignMulipleImpulseJoints { joints } => {
                    for joint in joints {
                        assign_single_joint(joint, &mut runners);
                    }
                }
                PartitionnerMessage::AssignImpulseJointTo { joint, target } => {
                    assign_single_joint(joint, &mut runners);
                }
                PartitionnerMessage::ReAssignImpulseJoint(joint) => {
                    assign_single_joint(joint, &mut runners);
                }
                PartitionnerMessage::AssignMultipleObjects { objects } => {
                    for object in objects {
                        assign_single_object(
                            object.uuid,
                            None,
                            object.aabb,
                            object.warm_object,
                            object.dynamic,
                            &mut runners,
                            &mut db,
                            curr_step,
                            is_running,
                        )?;
                    }
                }
                PartitionnerMessage::AssignIsland { origin, objects } => {
                    // Calculate the common target runner for al the objects.
                    if let Some(runner) = runners.runners.get_mut(&origin) {
                        // Remove previous assignment.
                        for object in &objects {
                            runner.assigned_objects.remove(&object.uuid);
                        }
                    }

                    let mut target_region = SimulationBounds::from_point(
                        objects[0].aabb.maxs,
                        SimulationBounds::DEFAULT_WIDTH,
                    );

                    for object in &objects {
                        let candidate_region = SimulationBounds::from_point(
                            object.aabb.maxs,
                            SimulationBounds::DEFAULT_WIDTH,
                        );

                        if candidate_region > target_region {
                            target_region = candidate_region;
                        }
                    }

                    for object in objects {
                        assign_object_to_region(
                            &zenoh,
                            &mut db,
                            &mut runners,
                            target_region,
                            object.uuid,
                            object.warm_object,
                            curr_step,
                            is_running,
                        )?;
                    }
                }
                PartitionnerMessage::AssignObjectTo {
                    uuid,
                    origin,
                    target,
                    warm_object,
                } => {
                    if let Some(runner) = runners.runners.get_mut(&origin) {
                        // Remove previous assignment.
                        runner.assigned_objects.remove(&uuid);
                    }

                    assign_object_to_region(
                        &zenoh,
                        &mut db,
                        &mut runners,
                        target,
                        uuid,
                        warm_object,
                        curr_step,
                        is_running,
                    )?;
                }
                PartitionnerMessage::ReAssignObject {
                    uuid,
                    origin,
                    aabb,
                    warm_object,
                    dynamic,
                } => assign_single_object(
                    uuid,
                    Some(origin),
                    aabb,
                    warm_object,
                    dynamic,
                    &mut runners,
                    &mut db,
                    curr_step,
                    is_running,
                )?,
                PartitionnerMessage::MoveObject { uuid, position } => {
                    // Broadcast the message to all the runners dealing with this object.
                    let runner_message = RunnerMessage::MoveObject { uuid, position };
                    for (_, runner) in &mut runners.runners {
                        if runner.assigned_objects.contains(&uuid) {
                            runner.pending.push(runner_message.clone());
                            runner.send_pending_if_started(&zenoh);
                        }
                    }
                }
                PartitionnerMessage::UpdateColdObject { uuid } => {
                    // Broadcast the message to all the runners dealing with this object.
                    let runner_message = RunnerMessage::UpdateColdObject { uuid };
                    for (_, runner) in &mut runners.runners {
                        if runner.assigned_objects.contains(&uuid) {
                            runner.pending.push(runner_message.clone());
                            runner.send_pending_if_started(&zenoh);
                        }
                    }
                }
                PartitionnerMessage::RemoveObject => { /* TODO */ }
                PartitionnerMessage::AckStart { origin } => {
                    if let Some(runner) = runners.runners.get_mut(&origin) {
                        println!("Runner {origin} acked start.");
                        runner.started = true;
                        runner.send_pending_if_started(&zenoh);
                    }
                }
            }
        }

        let num_regions_after = runners.runners.len();

        if num_regions_after != num_regions_before {
            let region_list = RegionList {
                keys: runners.runners.values().map(|r| r.key.clone()).collect(),
                ports: runners.runners.values().map(|r| r.port).collect(),
            };
            db.put("region_list", &region_list);
        }
        if pending_acks == 0 && !runners.runners.is_empty() {
            let real_time_step_increments_duration = STEP_INCREMENTS as f32 * 0.016;
            if last_ack_time.elapsed().as_secs_f32() < real_time_step_increments_duration {
                std::thread::sleep(Duration::from_secs_f32(
                    real_time_step_increments_duration - last_ack_time.elapsed().as_secs_f32(),
                ));
            }
            last_ack_time = Instant::now();

            if is_running {
                for runner in runners.runners.values_mut() {
                    runner.pending.push(RunnerMessage::RunSteps {
                        curr_step,
                        num_steps: STEP_INCREMENTS,
                    });
                    runner.send_pending_if_started(&zenoh);
                    pending_acks += 1;
                }

                curr_step += STEP_INCREMENTS as u64;
            }
        }
    }

    Ok(())
}

// struct MongoDb {
//     client: mongodb::sync::Client,
//     database: mongodb::sync::Database,
//     counters: mongodb::sync::Collection,
// }
//
// impl MongoDb {
//     pub fn new() -> Self {
//         let mut client_options =
//             mongodb::options::ClientOptions::parse("mongodb://localhost:27017")?;
//         let client = mongodb::sync::Client::with_options(client_options)?;
//         let database = client.database("steadyum");
//         let counters = client.collection::<u32>();
//         Self {
//             client,
//             database,
//             counters,
//         }
//     }
//
//     pub fn reset_counter(&mut self) {
//         self.counters.find_one_and_delete(doc! {
//             { count },
//             None
//         })
//     }
//
//     pub fn inc_counter(&mut self) -> u32 {
//         let val = self.counters.find_one_and_update(
//             doc! {},
//             doc! {
//                 $inc: { count: 1 },
//                 {returnOriginal: false}
//             },
//             None,
//         );
//         val.unwrap().unwrap()
//     }
// }
