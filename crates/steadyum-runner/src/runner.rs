use crate::cli::CliArgs;
use crate::connected_components::calculate_connected_components;
use crate::neighbors::Neighbors;
use crate::region_assignment::{
    apply_and_send_region_assignments, calculate_region_assignments, RegionAssignments,
};
use crate::watch::{
    compute_watch_data, read_watched_objects, set_watched_sets, WatchedObject, MAIN_GROUP,
    WATCH_GROUP,
};
use log::info;
use rapier::data::Coarena;
use rapier::prelude::*;
use std::collections::HashMap;
use std::time::Duration;
use steadyum_api_types::kinematic::KinematicAnimations;
use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::messages::{AckSteps, BodyAssignment, RunnerMessage};
use steadyum_api_types::objects::{
    BodyPositionObject, ColdBodyObject, WarmBodyObject, WarmBodyObjectSet, WatchedObjects,
};
use steadyum_api_types::region_db::PartitionnerServer;
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::{runner_zenoh_ack_key, runner_zenoh_commands_key, ZenohContext};
use uuid::Uuid;
use zenoh::prelude::sync::SyncResolve;
use zenoh::prelude::SplitBuffer;

pub const NUM_INTERNAL_STEPS: u64 = 10;

#[derive(Default)]
pub struct SimulationState {
    pub step_id: u64,
    pub is_running: bool,
    pub query_pipeline: QueryPipeline,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub gravity: Vector<f32>,
    pub params: IntegrationParameters,
    pub islands: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub physics_pipeline: PhysicsPipeline,
    pub body2animations: Coarena<KinematicAnimations>,
    pub body2uuid: HashMap<RigidBodyHandle, Uuid>,
    pub uuid2body: HashMap<Uuid, RigidBodyHandle>,
    pub sim_bounds: SimulationBounds,
    pub watched_objects: HashMap<RigidBodyHandle, WatchedObject>,
}

#[derive(Copy, Clone, Debug, Default)]
struct MainLoopTimings {
    pub message_processing: f32,
    pub simulation_step: f32,
    pub connected_components: f32,
    pub data_and_watch_list: f32,
    pub release_reassign: f32,
    pub ack: f32,
}

pub fn run_simulation(args: CliArgs) -> anyhow::Result<()> {
    let mut kvs = KvsContext::new().expect("B");
    let my_uuid = args.typed_uuid();
    info!("My uuid: {:?}", my_uuid);
    let mut db = PartitionnerServer::new()?;

    let zenoh = ZenohContext::new().expect("Runner error 1");
    let mut neighbors = Neighbors::new(&zenoh);
    let mut sim_state = SimulationState::default();
    sim_state.is_running = false;
    sim_state.gravity = Vector::y() * (-9.81);

    // Subscribe to command queue.
    let runner_zenoh_key = runner_zenoh_commands_key(my_uuid);
    let runner_zenoh_commands_queue = zenoh
        .session
        .declare_subscriber(&runner_zenoh_key)
        .res_sync()
        .unwrap();

    let mut runner_ack_key = String::new();
    let mut runner_zenoh_ack_queue = None;

    // We started listening to the command queue, we can now register this runner as
    // ready to be assigned.
    db.put_runner_initialized(my_uuid)?;

    let mut watch_iteration_id = 0;
    let stopped = false;
    // NOTE: the step_id starts at least at 1 to avoid an underflow in the time sync check.
    sim_state.step_id = args.time_origin.max(1);
    assert!(sim_state.step_id >= 1);

    /*
     * Wait for region assignment (blocking).
     */
    let mut delayed_messages = vec![]; // Messages that we received before we got assigned a region.
    let mut pending_assignments = vec![];

    info!("[{}] waiting for messagse", my_uuid);

    while let Ok(sample) = runner_zenoh_commands_queue.recv() {
        let payload = sample.value.payload.contiguous();
        let body = String::from_utf8_lossy(&payload);
        let message: RunnerMessage = serde_json::from_str(&body).unwrap();

        match message {
            RunnerMessage::AssignRegion {
                region,
                time_origin,
            } => {
                info!("[{}] assigned region {:?}", my_uuid, region);
                sim_state.sim_bounds = region;
                sim_state.step_id = time_origin.max(1);
                // Setup the queue adjacent runners will subscribe to for time sync.
                runner_ack_key = runner_zenoh_ack_key(&region);
                runner_zenoh_ack_queue = Some(
                    zenoh
                        .session
                        .declare_publisher(&runner_ack_key)
                        .res_sync()
                        .unwrap(),
                );
                neighbors.init_neighbor_acks(sim_state.sim_bounds).unwrap();
                break;
            }
            // As long as we are not assigned to any region, other messages
            // will be processed later.
            _ => delayed_messages.push(message),
        }
    }

    /*
     * Process delayed messages.
     */

    // If we reach this point, we got a region assigned.
    for message in delayed_messages {
        process_message(my_uuid, &mut sim_state, message, &mut pending_assignments).unwrap();
    }

    /*
     * Main runner loop.
     * We will only reach this point if a region was assigned to us.
     */
    while !stopped {
        let mut timings = MainLoopTimings::default();
        let loop_time = std::time::Instant::now();

        let t0 = std::time::Instant::now();

        loop {
            // Process messages.
            while let Ok(sample) = runner_zenoh_commands_queue.try_recv() {
                let payload = sample.value.payload.contiguous();
                let body = String::from_utf8_lossy(&payload);
                println!("Message received: {}", body);
                let message: RunnerMessage = serde_json::from_str(&body).unwrap();
                process_message(my_uuid, &mut sim_state, message, &mut pending_assignments)?;
            }

            // Time sync.
            let timestamp = neighbors.update_neighbor_acks().unwrap();

            // info!(
            //     "[{}] neighbor timestamp: {:?}, my step id: {}",
            //     my_uuid, timestamp, sim_state.step_id
            // );

            if timestamp.unwrap_or(sim_state.step_id) >= sim_state.step_id - 1 {
                break;
            }
        }

        watch_iteration_id += 1;
        let watched = read_watched_objects(&mut kvs, sim_state.sim_bounds);
        // set_watched_sets(watched, &mut sim_state, watch_iteration_id);

        resolve_pending_assignments(&mut sim_state, &mut pending_assignments);

        let mut region_assignments = RegionAssignments::default();

        if sim_state.is_running {
            let t0 = std::time::Instant::now();

            for sub_step_id in 0..NUM_INTERNAL_STEPS {
                sim_state.physics_pipeline.step(
                    &sim_state.gravity,
                    &sim_state.params,
                    &mut sim_state.islands,
                    &mut sim_state.broad_phase,
                    &mut sim_state.narrow_phase,
                    &mut sim_state.bodies,
                    &mut sim_state.colliders,
                    &mut sim_state.impulse_joints,
                    &mut sim_state.multibody_joints,
                    &mut sim_state.ccd_solver,
                    None,
                    &(),
                    &(),
                );

                let current_physics_time =
                    (sim_state.step_id * NUM_INTERNAL_STEPS + sub_step_id + 1) as Real
                        * sim_state.params.dt;

                // Update animations.
                for (handle, animations) in sim_state.body2animations.iter() {
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

            timings.simulation_step = t0.elapsed().as_secs_f32();

            let t0 = std::time::Instant::now();
            let connected_components = calculate_connected_components(&sim_state);
            region_assignments = calculate_region_assignments(&sim_state, connected_components);
            timings.connected_components = t0.elapsed().as_secs_f32();
        }

        let num_steps_run = if sim_state.is_running {
            sim_state.step_id += 1;
            NUM_INTERNAL_STEPS
        } else {
            1
        };

        let t0 = std::time::Instant::now();
        let mut all_data = vec![];

        for (handle, body) in sim_state.bodies.iter() {
            if !sim_state.watched_objects.contains_key(&handle) {
                let warm_object =
                    WarmBodyObject::from_body(body, sim_state.step_id * NUM_INTERNAL_STEPS);
                let uuid = sim_state.body2uuid[&handle].clone();

                let pos_object = BodyPositionObject {
                    uuid,
                    timestamp: warm_object.timestamp,
                    position: warm_object.position,
                };
                all_data.push(pos_object);
            }
        }

        let mut watch_data = compute_watch_data(&sim_state, num_steps_run as usize);

        timings.data_and_watch_list = t0.elapsed().as_secs_f32();

        let t0 = std::time::Instant::now();

        apply_and_send_region_assignments(
            &mut sim_state,
            &region_assignments,
            &mut neighbors,
            &db,
        )?;

        /*
         * Upload the new positions for clients, as well as the watch set.
         */
        {
            let warm_set = WarmBodyObjectSet {
                timestamp: sim_state.step_id * NUM_INTERNAL_STEPS,
                objects: all_data,
            };
            kvs.put_warm(&sim_state.sim_bounds.runner_key(), &warm_set)
                .expect("C");
            kvs.put(
                &sim_state.sim_bounds.watch_kvs_key(),
                &WatchedObjects {
                    objects: watch_data,
                },
            )
            .expect("D");
        }

        timings.release_reassign = t0.elapsed().as_secs_f32();

        // println!("{} steps to run: {}", runner_zenoh_key, sim_state.steps_to_run);

        let t0 = std::time::Instant::now();

        {
            // Send the ack.
            let ack = AckSteps {
                step_id: sim_state.step_id,
            };
            let ack_str = serde_json::to_string(&ack).unwrap();
            runner_zenoh_ack_queue
                .as_ref()
                .unwrap()
                .put(ack_str)
                .res_sync()
                .unwrap();
        }
        timings.ack = t0.elapsed().as_secs_f32();

        let elapsed = loop_time.elapsed().as_secs_f32();
        let time_limit = num_steps_run as Real * sim_state.params.dt;
        if elapsed < time_limit / 2.0 {
            std::thread::sleep(Duration::from_secs_f32(time_limit - elapsed));
        }
    }

    Ok(())
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

fn resolve_pending_assignments(
    sim_state: &mut SimulationState,
    pending_assignments: &mut Vec<BodyAssignment>,
) {
    pending_assignments.retain(|data| {
        if data.warm.timestamp > sim_state.step_id {
            println!("{} > {}", data.warm.timestamp, sim_state.step_id);
            // This body lives in the future, we can’t simulate it for now.
            return true;
        }

        if let Some(handle) = sim_state.uuid2body.get(&data.uuid) {
            sim_state.bodies.remove(
                *handle,
                &mut sim_state.islands,
                &mut sim_state.colliders,
                &mut sim_state.impulse_joints,
                &mut sim_state.multibody_joints,
                true,
            );
            sim_state.watched_objects.remove(handle);
        }

        let (body, collider) = make_builders(&data.cold, data.warm);
        let watch_shape_radius = collider.shape.compute_local_bounding_sphere().radius * 1.1;
        let body_handle = sim_state.bodies.insert(body);
        sim_state
            .colliders
            .insert_with_parent(collider, body_handle, &mut sim_state.bodies);
        let watch_collider = ColliderBuilder::ball(watch_shape_radius)
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
            .insert_with_parent(watch_collider, body_handle, &mut sim_state.bodies);
        sim_state.body2uuid.insert(body_handle, data.uuid.clone());
        sim_state.uuid2body.insert(data.uuid, body_handle);
        sim_state
            .body2animations
            .insert(body_handle.0, data.cold.animations.clone());

        // for data in impulse_joints {
        //     if let (Some(handle1), Some(handle2)) = (
        //         sim_state.uuid2body.get(&data.body1),
        //         sim_state.uuid2body.get(&data.body2),
        //     ) {
        //         sim_state
        //             .impulse_joints
        //             .insert(*handle1, *handle2, data.joint, true);
        //     }
        // }

        false
    });
}

fn process_message(
    my_uuid: Uuid,
    sim_state: &mut SimulationState,
    message: RunnerMessage,
    pending_assignments: &mut Vec<BodyAssignment>,
) -> anyhow::Result<()> {
    match message {
        RunnerMessage::AssignRegion {
            region,
            time_origin,
        } => {
            sim_state.sim_bounds = region;
            sim_state.step_id = time_origin;
        }
        RunnerMessage::AssignIsland {
            mut bodies,
            impulse_joints,
        } => {
            info!(
                "[{}] adding {} bodies and {} imp. joints",
                my_uuid,
                bodies.len(),
                impulse_joints.len()
            );
            pending_assignments.append(&mut bodies);
        }
        RunnerMessage::MoveObject { .. } => {
            /*
            if let Some(handle) = sim_state.uuid2body.get(&uuid) {
                if let Some(rb) = sim_state.bodies.get_mut(*handle) {
                    rb.set_position(position, true);
                }
            }
             */
        }
        RunnerMessage::UpdateColdObject { .. } => {
            /*
            if let Ok(cold_object) = kvs.get_cold_object(uuid) {
                if let Some(handle) = sim_state.uuid2body.get(&uuid) {
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
             */
        }
        RunnerMessage::StartStop { running } => sim_state.is_running = running,
    }

    Ok(())
}
