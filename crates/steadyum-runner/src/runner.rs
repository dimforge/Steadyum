use crate::cli::CliArgs;
use crate::watch::read_watched_objects;
use clap::arg;
use flume::{Receiver, Sender};
use rapier::data::Coarena;
use rapier::parry::bounding_volume::BoundingSphere;
use rapier::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use steadyum_api_types::kinematic::KinematicAnimations;
use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::messages::{
    ImpulseJointAssignment, ObjectAssignment, PartitionnerMessage, PARTITIONNER_QUEUE,
};
use steadyum_api_types::objects::{
    BodyPositionObject, ColdBodyObject, WarmBodyObject, WarmBodyObjectSet, WatchedObjects,
};
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::{put_json, ZenohContext};
use uuid::Uuid;
use zenoh::prelude::sync::SyncResolve;

const WATCH_GROUP: Group = Group::GROUP_1;
const MAIN_GROUP: Group = Group::GROUP_2;

pub struct SimulationState {
    pub query_pipeline: QueryPipeline,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub body2uuid: HashMap<RigidBodyHandle, Uuid>,
}

#[derive(Clone)]
pub struct SharedSimulationState(pub Arc<RwLock<SimulationState>>);

#[derive(Copy, Clone, Debug, Default)]
struct MainLoopTimings {
    pub message_processing: f32,
    pub simulation_step: f32,
    pub connected_components: f32,
    pub data_and_watch_list: f32,
    pub release_reassign: f32,
    pub ack: f32,
}

pub enum RunnerCommand {
    AssignJoint(ImpulseJointAssignment),
    CreateBody {
        uuid: Uuid,
        cold_object: ColdBodyObject,
        warm_object: WarmBodyObject,
    },
    MoveBody {
        uuid: Uuid,
        position: Isometry<Real>,
    },
    UpdateColdObject {
        uuid: Uuid,
    },
    SetWatchedSet(Vec<(WatchedObjects, SimulationBounds)>),
    StartStop {
        running: bool,
    },
    RunSteps {
        curr_step: u64,
        num_steps: u32,
    },
}

struct WatchedObject {
    region: SimulationBounds,
    watch_iteration_id: usize,
}

impl WatchedObject {
    pub fn new(region: SimulationBounds, watch_iteration_id: usize) -> Self {
        Self {
            region,
            watch_iteration_id,
        }
    }
}

pub fn spawn_simulation(
    args: CliArgs,
    commands: Receiver<RunnerCommand>,
    commands_snd: Sender<RunnerCommand>,
) -> SharedSimulationState {
    let sim_bounds = args.simulation_bounds();
    let sim_state = SimulationState {
        query_pipeline: QueryPipeline::new(),
        bodies: RigidBodySet::new(),
        colliders: ColliderSet::new(),
        body2uuid: Default::default(),
    };

    let shared_state = SharedSimulationState(Arc::new(RwLock::new(sim_state)));
    let returned_shared_state = shared_state.clone();

    std::thread::spawn(move || {
        let runner_key = sim_bounds.runner_key();
        let mut kvs = KvsContext::new().expect("B");

        let zenoh = ZenohContext::new().expect("Runner error 1");
        let partitionner = zenoh
            .session
            .declare_publisher(PARTITIONNER_QUEUE)
            .res_sync()
            .expect("Runner error 2");
        let gravity = Vector::y() * (-9.81);
        let params = IntegrationParameters::default();
        let mut islands = IslandManager::new();
        let mut broad_phase = BroadPhase::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut impulse_joints = ImpulseJointSet::new();
        let mut multibody_joints = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();
        let mut physics_pipeline = PhysicsPipeline::new();
        let mut uuid2body: HashMap<Uuid, RigidBodyHandle> = HashMap::default();
        let mut body2animations = Coarena::<KinematicAnimations>::new();
        let runner_zenoh_key = args.simulation_bounds().zenoh_queue_key();
        let my_watch_key = sim_bounds.watch_kvs_key();
        let mut is_running = false;
        let mut watched_objects = HashMap::new();

        let mut prev_pos = HashMap::new();
        let (coll_events_snd, coll_events_rcv) = rapier::crossbeam::channel::unbounded();
        let (force_events_snd, force_events_rcv) = rapier::crossbeam::channel::unbounded();
        let events = ChannelEventCollector::new(coll_events_snd, force_events_snd);

        let mut watch_iteration_id = 0;
        let mut step_id = args.time_origin;
        let mut steps_to_run = 0;
        let mut stopped = false;

        while !stopped {
            let mut timings = MainLoopTimings::default();
            let loop_time = std::time::Instant::now();
            watch_iteration_id += 1;

            let t0 = std::time::Instant::now();

            let mut sim_state_guard = shared_state.0.write().expect("A");
            let mut sim_state = &mut *sim_state_guard;

            while let Ok(command) = commands.try_recv() {
                match command {
                    RunnerCommand::RunSteps {
                        curr_step,
                        num_steps,
                    } => {
                        step_id = curr_step;
                        steps_to_run = num_steps;

                        // Read the latest watched sets.
                        let watched = read_watched_objects(&mut kvs, sim_bounds);
                        set_watched_set(
                            watched,
                            &mut uuid2body,
                            &mut watched_objects,
                            &mut sim_state,
                            &mut islands,
                            &mut impulse_joints,
                            &mut multibody_joints,
                            watch_iteration_id,
                        );

                        // All messages received after the RunStep have to be processed at the next step
                        // to avoid, e.g., double integration of the same body.
                        break;
                    }
                    RunnerCommand::AssignJoint(joint_assignment) => {
                        if let (Some(handle1), Some(handle2)) = (
                            uuid2body.get(&joint_assignment.body1),
                            uuid2body.get(&joint_assignment.body2),
                        ) {
                            impulse_joints.insert(*handle1, *handle2, joint_assignment.joint, true);
                        }
                    }
                    RunnerCommand::CreateBody {
                        uuid,
                        cold_object,
                        warm_object,
                    } => {
                        if let Some(handle) = uuid2body.get(&uuid) {
                            sim_state.bodies.remove(
                                *handle,
                                &mut islands,
                                &mut sim_state.colliders,
                                &mut impulse_joints,
                                &mut multibody_joints,
                                true,
                            );
                            watched_objects.remove(handle);
                        }

                        // if step_id < warm_object.timestamp {
                        //     println!(
                        //         "Object from the future: {} vs {}",
                        //         step_id, warm_object.timestamp
                        //     );
                        // }
                        let (body, collider) = make_builders(&cold_object, warm_object);
                        let watch_shape_radius =
                            collider.shape.compute_local_bounding_sphere().radius * 1.1;
                        let body_handle = sim_state.bodies.insert(body);
                        let co_handle = sim_state.colliders.insert_with_parent(
                            collider,
                            body_handle,
                            &mut sim_state.bodies,
                        );
                        let watch_collider = ColliderBuilder::ball(watch_shape_radius)
                            .density(0.0)
                            .collision_groups(InteractionGroups::new(
                                // We don’t care about watched objects intersecting each others.
                                WATCH_GROUP,
                                MAIN_GROUP,
                            ))
                            // Watched objects don’t generate forces.
                            .solver_groups(InteractionGroups::none());
                        sim_state.colliders.insert_with_parent(
                            watch_collider,
                            body_handle,
                            &mut sim_state.bodies,
                        );
                        sim_state.body2uuid.insert(body_handle, uuid.clone());
                        uuid2body.insert(uuid, body_handle);
                        body2animations.insert(body_handle.0, cold_object.animations);
                    }
                    RunnerCommand::MoveBody { uuid, position } => {
                        if let Some(handle) = uuid2body.get(&uuid) {
                            if let Some(rb) = sim_state.bodies.get_mut(*handle) {
                                rb.set_position(position, true);
                            }
                        }
                    }
                    RunnerCommand::UpdateColdObject { uuid } => {
                        if let Ok(cold_object) = kvs.get_cold_object(uuid) {
                            if let Some(handle) = uuid2body.get(&uuid) {
                                if let Some(rb) = sim_state.bodies.get_mut(*handle) {
                                    if cold_object.body_type == RigidBodyType::Fixed
                                        && rb.body_type() == RigidBodyType::Dynamic
                                    {
                                        let co = &sim_state.colliders[rb.colliders()[0]];
                                        // Broadcast the body to all the regions it intersects.
                                        let message = PartitionnerMessage::ReAssignObject {
                                            uuid,
                                            origin: runner_key.clone(),
                                            aabb: co.compute_aabb(),
                                            warm_object: WarmBodyObject::from_body(rb, step_id),
                                            dynamic: false,
                                        };
                                        put_json(&partitionner, &message);
                                    }

                                    rb.set_body_type(cold_object.body_type, true);
                                }
                            }
                        }
                    }
                    RunnerCommand::StartStop { running } => is_running = running,
                    RunnerCommand::SetWatchedSet(watched) => {
                        set_watched_set(
                            watched,
                            &mut uuid2body,
                            &mut watched_objects,
                            &mut sim_state,
                            &mut islands,
                            &mut impulse_joints,
                            &mut multibody_joints,
                            watch_iteration_id,
                        );
                    }
                }
            }
            timings.message_processing = t0.elapsed().as_secs_f32();

            if steps_to_run == 0 {
                continue;
            }

            let mut bodies_to_release = vec![];
            let mut joints_to_release = vec![];
            let mut bodies_to_reassign = vec![];
            let mut joints_to_reassign = vec![];
            let mut num_steps_run = 0;

            if is_running {
                let t0 = std::time::Instant::now();

                while steps_to_run > 0 {
                    physics_pipeline.step(
                        &gravity,
                        &params,
                        &mut islands,
                        &mut broad_phase,
                        &mut narrow_phase,
                        &mut sim_state.bodies,
                        &mut sim_state.colliders,
                        &mut impulse_joints,
                        &mut multibody_joints,
                        &mut ccd_solver,
                        None,
                        &(),
                        &(),
                    );
                    step_id += 1;
                    steps_to_run -= 1;
                    num_steps_run += 1;

                    let current_physics_time = step_id as Real * params.dt;

                    // Update animations.
                    for (handle, animations) in body2animations.iter() {
                        if animations.linear.is_none() && animations.angular.is_none() {
                            // Nothing to animate.
                            continue;
                        }

                        // println!("Animating: {:?}.", handle);
                        if let Some(rb) = sim_state.bodies.get_mut(RigidBodyHandle(handle)) {
                            let new_pos = animations.eval(current_physics_time, *rb.position());
                            // TODO: what if it’s a velocity-based kinematic body?
                            // println!("prev: {:?}, new: {:?}", rb.position(), new_pos);
                            rb.set_next_kinematic_position(new_pos);
                        }
                    }
                }

                // for (h, body) in sim_state.bodies.iter() {
                //     if body.is_dynamic() && !watched_objects.contains_key(&h) {
                //         println!(
                //             "[{}] {} Linvel: {}, angvel: {}",
                //             runner_key,
                //             step_id,
                //             body.linvel().norm(),
                //             body.angvel().norm(),
                //         );
                //     }
                // }

                timings.simulation_step = t0.elapsed().as_secs_f32();
                // println!("Simulation step: {}", timings.simulation_step);
                // println!("Num joints: {}", impulse_joints.len());

                let t0 = std::time::Instant::now();
                let connected_components = calculate_connected_components(
                    &sim_state.bodies,
                    &sim_state.colliders,
                    &impulse_joints,
                    &narrow_phase,
                );

                'next_cc: for connected_component in connected_components {
                    let mut island_is_in_smaller_region = true;

                    // See if the island hit an object in a master region.
                    for handle in &connected_component.bodies {
                        if let Some(watched) = watched_objects.get(handle) {
                            bodies_to_reassign.extend(
                                connected_component
                                    .bodies
                                    .iter()
                                    .filter(|h| !watched_objects.contains_key(&h))
                                    .map(|h| (*h, watched.region)),
                            );
                            joints_to_reassign.extend(
                                connected_component
                                    .joints
                                    .iter()
                                    .map(|j| (*j, watched.region)),
                            );
                            continue 'next_cc;
                        }
                    }

                    // See if any object in the island entered a master region.
                    let mut best_region = sim_bounds;
                    for handle in &connected_component.bodies {
                        let body = &sim_state.bodies[*handle];
                        let aabb = sim_state.colliders[body.colliders()[0]].compute_aabb();

                        // Check if the body should switch region.
                        let region =
                            SimulationBounds::from_aabb(&aabb, SimulationBounds::DEFAULT_WIDTH);
                        if region > best_region {
                            best_region = region;
                        }
                    }

                    if best_region > sim_bounds {
                        bodies_to_reassign.extend(
                            connected_component
                                .bodies
                                .iter()
                                .filter(|h| !watched_objects.contains_key(&h))
                                .map(|h| (*h, best_region)),
                        );
                        joints_to_reassign
                            .extend(connected_component.joints.iter().map(|j| (*j, best_region)));
                        continue 'next_cc;
                    }

                    for handle in &connected_component.bodies {
                        // See if we should release ownership to a child region.
                        // (check that none of the bodies in the connected-component belong
                        //  to this region).
                        if let Some(aabb) = sim_state
                            .bodies
                            .get(*handle)
                            .and_then(|rb| sim_state.colliders.get(rb.colliders()[0]))
                            .map(|co| co.compute_aabb())
                        {
                            island_is_in_smaller_region = island_is_in_smaller_region
                                && sim_bounds.is_in_smaller_region(&aabb);
                        }
                    }

                    if island_is_in_smaller_region {
                        bodies_to_release.push(connected_component.bodies);
                        joints_to_release.extend_from_slice(&connected_component.joints);
                    }
                }
                timings.connected_components = t0.elapsed().as_secs_f32();
            } else {
                steps_to_run = 0;
            }

            let t0 = std::time::Instant::now();
            let mut all_data = vec![];
            let mut watch_data = vec![];

            for (handle, body) in sim_state.bodies.iter() {
                if !watched_objects.contains_key(&handle) {
                    let warm_object = WarmBodyObject::from_body(body, step_id);
                    let uuid = sim_state.body2uuid[&handle].clone();

                    let prev_pos = prev_pos.entry(handle).or_insert(Isometry::identity());
                    let delta_pos = prev_pos.inverse() * warm_object.position;

                    if true
                    // || !body.is_sleeping()
                    //     && (delta_pos.translation.vector.norm() > 1.0e-3
                    //         || delta_pos.rotation.coords.xyz().norm() > 1.0e-2)
                    {
                        let pos_object = BodyPositionObject {
                            uuid,
                            timestamp: warm_object.timestamp,
                            position: warm_object.position,
                        };
                        all_data.push(pos_object);
                        *prev_pos = warm_object.position;
                    }

                    let predicted_pos = body.predict_position_using_velocity_and_forces(
                        params.dt * num_steps_run as f32,
                    );
                    let aabb =
                        sim_state.colliders[body.colliders()[0]].compute_swept_aabb(&predicted_pos);
                    let sphere =
                        BoundingSphere::new(aabb.center(), aabb.half_extents().norm() * 1.1);
                    watch_data.push((uuid, sphere));
                }
            }
            timings.data_and_watch_list = t0.elapsed().as_secs_f32();

            let t0 = std::time::Instant::now();
            // println!(
            //     "To release: {}, to reassign: {}",
            //     bodies_to_release.len(),
            //     bodies_to_reassign.len()
            // );
            for handles in &bodies_to_release {
                let objects: Vec<_> = handles
                    .iter()
                    .map(|handle| {
                        let body = &sim_state.bodies[*handle];
                        let warm_object = WarmBodyObject::from_body(body, step_id);

                        // Switch region.
                        let collider = &sim_state.colliders[body.colliders()[0]];
                        let aabb = collider.compute_aabb();
                        ObjectAssignment {
                            uuid: sim_state.body2uuid[&handle].clone(),
                            aabb,
                            warm_object,
                            dynamic: true,
                        }
                    })
                    .collect();

                let partitioner_message = PartitionnerMessage::AssignIsland {
                    origin: runner_zenoh_key.clone(),
                    objects,
                };

                put_json(&partitionner, &partitioner_message);
            }

            for (h1, h2, joint, joint_handle) in &joints_to_release {
                let assignment = ImpulseJointAssignment {
                    body1: sim_state.body2uuid[h1].clone(),
                    body2: sim_state.body2uuid[h2].clone(),
                    joint: *joint,
                };

                let partitionner_message = PartitionnerMessage::ReAssignImpulseJoint(assignment);
                put_json(&partitionner, &partitionner_message);
                impulse_joints.remove(*joint_handle, true);
            }

            for (handle, new_region) in &bodies_to_reassign {
                let body = &sim_state.bodies[*handle];
                let warm_object = WarmBodyObject::from_body(body, step_id);

                // Switch region.
                let partitionner_message = PartitionnerMessage::AssignObjectTo {
                    uuid: sim_state.body2uuid[&handle].clone(),
                    origin: runner_zenoh_key.clone(),
                    target: *new_region,
                    warm_object,
                };
                put_json(&partitionner, &partitionner_message);
            }

            for ((h1, h2, joint, joint_handle), new_region) in &joints_to_reassign {
                let assignment = ImpulseJointAssignment {
                    body1: sim_state.body2uuid[h1].clone(),
                    body2: sim_state.body2uuid[h2].clone(),
                    joint: *joint,
                };

                let partitionner_message = PartitionnerMessage::AssignImpulseJointTo {
                    target: *new_region,
                    joint: assignment,
                };
                put_json(&partitionner, &partitionner_message);
                impulse_joints.remove(*joint_handle, true);
            }

            for handle in bodies_to_release
                .iter()
                .flat_map(|bodies| bodies.iter())
                .chain(bodies_to_reassign.iter().map(|(h, _)| h))
            {
                sim_state.bodies.remove(
                    *handle,
                    &mut islands,
                    &mut sim_state.colliders,
                    &mut impulse_joints,
                    &mut multibody_joints,
                    true,
                );
                if let Some(uuid) = sim_state.body2uuid.remove(handle) {
                    uuid2body.remove(&uuid);
                }
            }

            // steps_to_run -= 1;

            if steps_to_run == 0 {
                let warm_set = WarmBodyObjectSet {
                    timestamp: step_id,
                    objects: all_data,
                };
                kvs.put_warm(&runner_key, &warm_set).expect("C");
                kvs.put(
                    &sim_bounds.watch_kvs_key(),
                    &WatchedObjects {
                        objects: watch_data,
                    },
                )
                .expect("D");
            }
            timings.release_reassign = t0.elapsed().as_secs_f32();

            // println!("{} steps to run: {}", runner_zenoh_key, steps_to_run);

            let t0 = std::time::Instant::now();

            if steps_to_run == 0 {
                // Update the "stopped" flag.
                // stopped = true;
                // for (handle, body) in sim_state.bodies.iter() {
                //     if !watched_objects.contains_key(&handle) {
                //         stopped = false;
                //         break;
                //     }
                // }

                // Send the ack.
                let partitionner_message = &PartitionnerMessage::AckSteps {
                    origin: runner_zenoh_key.clone(),
                    stopped,
                };
                put_json(&partitionner, &partitionner_message);
            }
            timings.ack = t0.elapsed().as_secs_f32();

            if stopped {
                std::process::abort();
            }

            // if loop_time.elapsed().as_secs_f32() > 0.016 {
            //     println!(
            //         "Loop all time: {}, steps to run: {}, details: {:?}",
            //         loop_time.elapsed().as_secs_f32(),
            //         steps_to_run,
            //         timings
            //     );
            // }

            let elapsed = loop_time.elapsed().as_secs_f32();
            let time_limit = num_steps_run.max(1) as Real * params.dt;
            if elapsed < time_limit / 2.0 {
                std::thread::sleep(Duration::from_secs_f32(time_limit - elapsed));
            }
        }
    });

    returned_shared_state
}

fn make_builders(
    cold_object: &ColdBodyObject,
    warm_object: WarmBodyObject,
) -> (RigidBodyBuilder, ColliderBuilder) {
    let body = RigidBodyBuilder::new(cold_object.body_type)
        .position(warm_object.position)
        .linvel(warm_object.linvel)
        .angvel(warm_object.angvel)
        .can_sleep(false);
    let collider = ColliderBuilder::new(cold_object.shape.clone());
    (body, collider)
}

#[derive(Default, Clone)]
struct ConnectedComponent {
    bodies: Vec<RigidBodyHandle>,
    joints: Vec<(
        RigidBodyHandle,
        RigidBodyHandle,
        GenericJoint,
        ImpulseJointHandle,
    )>,
}

fn calculate_connected_components(
    bodies: &RigidBodySet,
    colliders: &ColliderSet,
    impulse_joints: &ImpulseJointSet,
    narrow_phase: &NarrowPhase,
) -> Vec<ConnectedComponent> {
    let mut visited = HashSet::new();
    let mut visited_joints = HashSet::new();
    let mut stack = vec![];
    let mut connected_bodies = vec![];
    let mut connected_joints = vec![];
    let mut result = vec![];

    for (handle, body) in bodies.iter() {
        stack.push(handle);

        let mut island_center = Point::origin();

        while let Some(body_handle) = stack.pop() {
            if visited.contains(&body_handle) {
                continue;
            }

            let body = &bodies[body_handle];
            if !body.is_dynamic() {
                continue;
            }

            island_center += body.center_of_mass().coords;
            visited.insert(body_handle);
            connected_bodies.push(body_handle);

            for collider_handle in bodies[body_handle].colliders() {
                for contact in narrow_phase.contacts_with(*collider_handle) {
                    let other_collider_handle = if contact.collider1 == *collider_handle {
                        contact.collider2
                    } else {
                        contact.collider1
                    };

                    if let Some(parent_handle) = colliders[other_collider_handle].parent() {
                        stack.push(parent_handle);
                    }
                }
            }

            for (rb1, rb2, joint_handle, joint) in impulse_joints.attached_joints(body_handle) {
                let other_body_handle = if rb1 == body_handle { rb2 } else { rb1 };

                if visited_joints.insert(joint_handle) {
                    connected_joints.push((rb1, rb2, joint.data, joint_handle));
                }
                stack.push(other_body_handle);
            }
        }

        let cc = ConnectedComponent {
            bodies: std::mem::replace(&mut connected_bodies, vec![]),
            joints: std::mem::replace(&mut connected_joints, vec![]),
        };

        if !cc.bodies.is_empty() || !cc.joints.is_empty() {
            result.push(cc);
        }
    }

    result
}

fn set_watched_set(
    watched: Vec<(WatchedObjects, SimulationBounds)>,
    uuid2body: &mut HashMap<Uuid, RigidBodyHandle>,
    watched_objects: &mut HashMap<RigidBodyHandle, WatchedObject>,
    sim_state: &mut SimulationState,
    islands: &mut IslandManager,
    impulse_joints: &mut ImpulseJointSet,
    multibody_joints: &mut MultibodyJointSet,
    watch_iteration_id: usize,
) {
    for (watched, region) in watched {
        for (uuid, bsphere) in watched.objects {
            let shape = SharedShape::ball(bsphere.radius);

            let rb_handle = if let Some((rb_handle, body)) = uuid2body
                .get(&uuid)
                .and_then(|h| sim_state.bodies.get_mut(*h).map(|rb| (h, rb)))
            {
                if !watched_objects.contains_key(rb_handle) {
                    continue;
                }

                let co_handle = body.colliders()[0];
                let collider = &mut sim_state.colliders[co_handle];
                collider.set_shape(shape);
                body.set_translation(bsphere.center().coords, false);
                *rb_handle
            } else {
                // Create the object to watch.
                let body = RigidBodyBuilder::dynamic().translation(bsphere.center().coords);
                let rb_handle = sim_state.bodies.insert(body);
                let collider = ColliderBuilder::new(shape)
                    .density(0.0)
                    .collision_groups(InteractionGroups::new(
                        // We don’t care about watched objects intersecting each others.
                        WATCH_GROUP,
                        MAIN_GROUP,
                    ))
                    // Watched objects don’t generate forces.
                    .solver_groups(InteractionGroups::none());
                sim_state
                    .colliders
                    .insert_with_parent(collider, rb_handle, &mut sim_state.bodies);
                uuid2body.insert(uuid.clone(), rb_handle);
                sim_state.body2uuid.insert(rb_handle, uuid);
                rb_handle
            };

            watched_objects.insert(rb_handle, WatchedObject::new(region, watch_iteration_id));
        }
    }

    // Remove all obsolete watched objects.
    watched_objects.retain(|handle, watched| {
        if watched.watch_iteration_id != watch_iteration_id {
            sim_state.bodies.remove(
                *handle,
                islands,
                &mut sim_state.colliders,
                impulse_joints,
                multibody_joints,
                true,
            );
            if let Some(uuid) = sim_state.body2uuid.remove(handle) {
                uuid2body.remove(&uuid);
            }
        }

        watched.watch_iteration_id == watch_iteration_id
    });
}
