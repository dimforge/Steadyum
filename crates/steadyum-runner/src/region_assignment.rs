use crate::connected_components::ConnectedComponent;
use crate::neighbors::Neighbors;
use crate::runner::SimulationState;
use crate::watch::WatchedObject;
use rapier::prelude::*;
use std::collections::HashMap;
use steadyum_api_types::messages::{
    BodyAssignment, ObjectAssignment, PartitionnerMessage, RunnerMessage,
};
use steadyum_api_types::objects::{ColdBodyObject, WarmBodyObject};
use steadyum_api_types::region_db::PartitionnerServer;
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::put_json;

#[derive(Default)]
pub struct RegionAssignments {
    bodies_to_reassign: HashMap<SimulationBounds, Vec<RigidBodyHandle>>,
}

pub fn calculate_region_assignments(
    sim_state: &SimulationState,
    connected_components: Vec<ConnectedComponent>,
) -> RegionAssignments {
    let mut result = RegionAssignments::default();

    'next_cc: for connected_component in connected_components {
        if connected_component.bodies.is_empty() {
            continue;
        }

        let mut best_region = SimulationBounds::smallest();

        for handle in &connected_component.bodies {
            let candidate_region = sim_state
                .watched_objects
                .get(handle)
                .map(|watched| watched.region)
                .unwrap_or_else(|| {
                    let body = &sim_state.bodies[*handle];
                    let aabb = sim_state.colliders[body.colliders()[0]].compute_aabb();
                    SimulationBounds::from_aabb(&aabb, SimulationBounds::DEFAULT_WIDTH)
                });

            if candidate_region > best_region {
                best_region = candidate_region;
            }
        }

        if best_region != sim_state.sim_bounds {
            let region = result
                .bodies_to_reassign
                .entry(best_region)
                .or_insert_with(Vec::new);
            region.extend(
                connected_component
                    .bodies
                    .iter()
                    .filter(|h| !sim_state.watched_objects.contains_key(&h))
                    .copied(),
            );
        }
    }

    result
}

pub fn apply_and_send_region_assignments(
    sim_state: &mut SimulationState,
    assignments: &RegionAssignments,
    neighbors: &mut Neighbors,
    db_context: &PartitionnerServer,
) -> anyhow::Result<()> {
    for (new_region, handles) in &assignments.bodies_to_reassign {
        if handles.is_empty() {
            continue;
        }

        let neighbor = neighbors.fetch_or_spawn_neighbor(db_context, *new_region);

        let body_assignments = handles
            .iter()
            .map(|handle| {
                let body = &sim_state.bodies[*handle];
                let collider = &sim_state.colliders[body.colliders()[0]];
                let uuid = sim_state.body2uuid[handle];
                let warm = WarmBodyObject::from_body(body, sim_state.step_id);
                let cold = ColdBodyObject::from_body_collider(body, collider);
                BodyAssignment { uuid, warm, cold }
            })
            .collect();

        // Switch region.
        let message = RunnerMessage::AssignIsland {
            bodies: body_assignments,
            impulse_joints: vec![],
        };

        neighbor.send(&message)?;
    }

    for handle in assignments
        .bodies_to_reassign
        .values()
        .flat_map(|handles| handles.iter())
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
