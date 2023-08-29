use crate::connected_components::ConnectedComponent;
use crate::neighbors::Neighbors;
use crate::runner::SimulationState;
use crate::watch::WatchedObject;
use rapier::prelude::*;
use std::collections::HashMap;
use steadyum_api_types::messages::{ObjectAssignment, PartitionnerMessage};
use steadyum_api_types::objects::WarmBodyObject;
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::put_json;

#[derive(Default)]
pub struct RegionAssignments {
    pub bodies_to_reassign: Vec<(RigidBodyHandle, SimulationBounds)>,
    pub bodies_to_release: Vec<Vec<RigidBodyHandle>>,
    /*
    pub joints_to_reassign: Vec<(ImpulseJointHandle, SimulationBounds)>,
    pub joints_to_release: Vec<
        Vec<(
            RigidBodyHandle,
            RigidBodyHandle,
            GenericJoint,
            ImpulseJointHandle,
        )>,
    >,
     */
}

pub fn calculate_region_assignments(
    sim_state: &SimulationState,
    connected_components: Vec<ConnectedComponent>,
    watched_objects: &HashMap<RigidBodyHandle, WatchedObject>,
) -> RegionAssignments {
    let mut result = RegionAssignments::default();

    'next_cc: for connected_component in connected_components {
        let mut island_is_in_smaller_region = true;

        // See if the island hit an object in a master region.
        for handle in &connected_component.bodies {
            if let Some(watched) = watched_objects.get(handle) {
                result.bodies_to_reassign.extend(
                    connected_component
                        .bodies
                        .iter()
                        .filter(|h| !watched_objects.contains_key(&h))
                        .map(|h| (*h, watched.region)),
                );
                // result.joints_to_reassign.extend(
                //     connected_component
                //         .joints
                //         .iter()
                //         .map(|j| (*j, watched.region)),
                // );
                continue 'next_cc;
            }
        }

        // See if any object in the island entered a master region.
        let mut best_region = sim_state.sim_bounds;
        for handle in &connected_component.bodies {
            let body = &sim_state.bodies[*handle];
            let aabb = sim_state.colliders[body.colliders()[0]].compute_aabb();

            // Check if the body should switch region.
            let region = SimulationBounds::from_aabb(&aabb, SimulationBounds::DEFAULT_WIDTH);
            if region > best_region {
                best_region = region;
            }
        }

        if best_region > sim_state.sim_bounds {
            result.bodies_to_reassign.extend(
                connected_component
                    .bodies
                    .iter()
                    .filter(|h| !watched_objects.contains_key(&h))
                    .map(|h| (*h, best_region)),
            );
            // result
            //     .joints_to_reassign
            //     .extend(connected_component.joints.iter().map(|j| (*j, best_region)));
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
                island_is_in_smaller_region =
                    island_is_in_smaller_region && sim_state.sim_bounds.is_in_smaller_region(&aabb);
            }
        }

        if island_is_in_smaller_region {
            result.bodies_to_release.push(connected_component.bodies);
            // result
            //     .joints_to_release
            //     .extend_from_slice(&connected_component.joints);
        }
    }

    result
}

pub fn apply_and_send_region_assignments(
    sim_state: &mut SimulationState,
    assignments: &RegionAssignments,
    neighbors: &Neighbors,
) -> anyhow::Result<()> {
    let runner_zenoh_key = sim_state.sim_bounds.zenoh_queue_key();

    for handles in &assignments.bodies_to_release {
        let objects: Vec<_> = handles
            .iter()
            .map(|handle| {
                let body = &sim_state.bodies[*handle];
                let warm_object = WarmBodyObject::from_body(body, sim_state.step_id);

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

        put_json(&neighbors.partitionner, &partitioner_message)?;
    }

    /*
    for (h1, h2, joint, joint_handle) in &assignments.joints_to_release {
        let assignment = ImpulseJointAssignment {
            body1: sim_state.body2uuid[h1].clone(),
            body2: sim_state.body2uuid[h2].clone(),
            joint: *joint,
        };

        let partitionner_message = PartitionnerMessage::ReAssignImpulseJoint(assignment);
        put_json(&partitionner, &partitionner_message)?;
        sim_state.impulse_joints.remove(*joint_handle, true);
    }
     */

    for (handle, new_region) in &assignments.bodies_to_reassign {
        let body = &sim_state.bodies[*handle];
        let warm_object = WarmBodyObject::from_body(body, sim_state.step_id);

        // Switch region.
        let partitionner_message = PartitionnerMessage::AssignObjectTo {
            uuid: sim_state.body2uuid[&handle].clone(),
            origin: runner_zenoh_key.clone(),
            target: *new_region,
            warm_object,
        };
        put_json(&neighbors.partitionner, &partitionner_message)?;
    }

    /*
    for ((h1, h2, joint, joint_handle), new_region) in &assignments.joints_to_reassign {
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
        sim_state.impulse_joints.remove(*joint_handle, true);
    }
     */

    for handle in assignments
        .bodies_to_release
        .iter()
        .flat_map(|bodies| bodies.iter())
        .chain(assignments.bodies_to_reassign.iter().map(|(h, _)| h))
    {
        sim_state.bodies.remove(
            *handle,
            &mut sim_state.islands,
            &mut sim_state.colliders,
            &mut sim_state.impulse_joints,
            &mut sim_state.multibody_joints,
            true,
        );
        if let Some(uuid) = sim_state.body2uuid.remove(handle) {
            sim_state.uuid2body.remove(&uuid);
        }
    }

    Ok(())
}
