use crate::cli::CliArgs;
use crate::connected_components::calculate_connected_components;
use crate::neighbors::Neighbors;
use crate::region_assignment::{
    apply_and_send_region_assignments, calculate_region_assignments, RegionAssignments,
};
use crate::watch::{
    compute_watch_data, read_watched_objects, set_watched_set, MAIN_GROUP, WATCH_GROUP,
};
use flume::Receiver;
use rapier::data::Coarena;
use rapier::parry::bounding_volume::BoundingSphere;
use rapier::prelude::*;
use std::collections::HashMap;
use std::time::Duration;
use steadyum_api_types::kinematic::KinematicAnimations;
use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::messages::{ImpulseJointAssignment, PartitionnerMessage};
use steadyum_api_types::objects::{
    BodyPositionObject, ColdBodyObject, WarmBodyObject, WarmBodyObjectSet, WatchedObjects,
};
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::{put_json, ZenohContext};
use uuid::Uuid;

#[derive(Default)]
pub struct SimulationState {
    pub step_id: u64,
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
    pub body2uuid: HashMap<RigidBodyHandle, Uuid>,
    pub uuid2body: HashMap<Uuid, RigidBodyHandle>,
    pub sim_bounds: SimulationBounds,
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
    StartStop {
        running: bool,
    },
    RunSteps {
        curr_step: u64,
        num_steps: u32,
    },
}

pub fn run_simulation(args: CliArgs, commands: Receiver<RunnerCommand>) -> anyhow::Result<()> {
    let sim_bounds = args.simulation_bounds();

    let runner_key = sim_bounds.runner_key();
    let mut kvs = KvsContext::new().expect("B");

    let zenoh = ZenohContext::new().expect("Runner error 1");
    let neighbors = Neighbors::new(&zenoh);
    let mut sim_state = SimulationState::default();
    sim_state.gravity = Vector::y() * (-9.81);

    let mut body2animations = Coarena::<KinematicAnimations>::new();
    let runner_zenoh_key = args.simulation_bounds().zenoh_queue_key();
    let mut is_running = false;
    let mut watched_objects = HashMap::new();

    let mut watch_iteration_id = 0;
    let mut steps_to_run = 0;
    let stopped = false;
    sim_state.step_id = args.time_origin;

    while !stopped {
        let mut timings = MainLoopTimings::default();
        let loop_time = std::time::Instant::now();
        watch_iteration_id += 1;

        let t0 = std::time::Instant::now();

        while let Ok(command) = commands.try_recv() {
            match command {
                RunnerCommand::RunSteps {
                    curr_step,
                    num_steps,
                } => {
                    sim_state.step_id = curr_step;
                    steps_to_run = num_steps;

                    // Read the latest watched sets.
                    let watched = read_watched_objects(&mut kvs, sim_bounds);
                    set_watched_set(
                        watched,
                        &mut watched_objects,
                        &mut sim_state,
                        watch_iteration_id,
                    );

                    // All messages received after the RunStep have to be processed at the next step
                    // to avoid, e.g., double integration of the same body.
                    break;
                }
                RunnerCommand::AssignJoint(joint_assignment) => {
                    if let (Some(handle1), Some(handle2)) = (
                        sim_state.uuid2body.get(&joint_assignment.body1),
                        sim_state.uuid2body.get(&joint_assignment.body2),
                    ) {
                        sim_state.impulse_joints.insert(
                            *handle1,
                            *handle2,
                            joint_assignment.joint,
                            true,
                        );
                    }
                }
                RunnerCommand::CreateBody {
                    uuid,
                    cold_object,
                    warm_object,
                } => {
                    if let Some(handle) = sim_state.uuid2body.get(&uuid) {
                        sim_state.bodies.remove(
                            *handle,
                            &mut sim_state.islands,
                            &mut sim_state.colliders,
                            &mut sim_state.impulse_joints,
                            &mut sim_state.multibody_joints,
                            true,
                        );
                        watched_objects.remove(handle);
                    }

                    let (body, collider) = make_builders(&cold_object, warm_object);
                    let watch_shape_radius =
                        collider.shape.compute_local_bounding_sphere().radius * 1.1;
                    let body_handle = sim_state.bodies.insert(body);
                    sim_state.colliders.insert_with_parent(
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
                    sim_state.uuid2body.insert(uuid, body_handle);
                    body2animations.insert(body_handle.0, cold_object.animations);
                }
                RunnerCommand::MoveBody { .. } => {
                    /*
                    if let Some(handle) = sim_state.uuid2body.get(&uuid) {
                        if let Some(rb) = sim_state.bodies.get_mut(*handle) {
                            rb.set_position(position, true);
                        }
                    }
                     */
                }
                RunnerCommand::UpdateColdObject { .. } => {
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
                RunnerCommand::StartStop { running } => is_running = running,
            }
        }
        timings.message_processing = t0.elapsed().as_secs_f32();

        if steps_to_run == 0 {
            continue;
        }

        let mut num_steps_run = 0;
        let mut region_assignments = RegionAssignments::default();

        if is_running {
            let t0 = std::time::Instant::now();

            while steps_to_run > 0 {
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
                sim_state.step_id += 1;
                steps_to_run -= 1;
                num_steps_run += 1;

                let current_physics_time = sim_state.step_id as Real * sim_state.params.dt;

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

            timings.simulation_step = t0.elapsed().as_secs_f32();

            let t0 = std::time::Instant::now();
            let connected_components = calculate_connected_components(&sim_state);
            region_assignments =
                calculate_region_assignments(&sim_state, connected_components, &watched_objects);
            timings.connected_components = t0.elapsed().as_secs_f32();
        } else {
            steps_to_run = 0;
        }

        let t0 = std::time::Instant::now();
        let mut all_data = vec![];

        for (handle, body) in sim_state.bodies.iter() {
            if !watched_objects.contains_key(&handle) {
                let warm_object = WarmBodyObject::from_body(body, sim_state.step_id);
                let uuid = sim_state.body2uuid[&handle].clone();

                let pos_object = BodyPositionObject {
                    uuid,
                    timestamp: warm_object.timestamp,
                    position: warm_object.position,
                };
                all_data.push(pos_object);
            }
        }

        let mut watch_data = compute_watch_data(&sim_state, &watched_objects, num_steps_run);

        timings.data_and_watch_list = t0.elapsed().as_secs_f32();

        let t0 = std::time::Instant::now();

        apply_and_send_region_assignments(&mut sim_state, &region_assignments, &neighbors)?;

        // steps_to_run -= 1;

        if steps_to_run == 0 {
            let warm_set = WarmBodyObjectSet {
                timestamp: sim_state.step_id,
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
            // Send the ack.
            let partitionner_message = &PartitionnerMessage::AckSteps {
                origin: runner_zenoh_key.clone(),
                stopped,
            };
            put_json(&neighbors.partitionner, &partitionner_message);
        }
        timings.ack = t0.elapsed().as_secs_f32();

        let elapsed = loop_time.elapsed().as_secs_f32();
        let time_limit = num_steps_run.max(1) as Real * sim_state.params.dt;
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
