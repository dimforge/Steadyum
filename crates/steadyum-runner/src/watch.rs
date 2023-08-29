use crate::runner::SimulationState;
use rapier::parry::bounding_volume::BoundingSphere;
use rapier::prelude::*;
use std::collections::HashMap;
use steadyum_api_types::kvs::KvsContext;
use steadyum_api_types::objects::{BodyPositionObject, WarmBodyObject, WatchedObjects};
use steadyum_api_types::simulation::SimulationBounds;
use uuid::Uuid;

pub const WATCH_GROUP: Group = Group::GROUP_1;
pub const MAIN_GROUP: Group = Group::GROUP_2;

pub struct WatchedObject {
    pub region: SimulationBounds,
    pub watch_iteration_id: usize,
}

impl WatchedObject {
    pub fn new(region: SimulationBounds, watch_iteration_id: usize) -> Self {
        Self {
            region,
            watch_iteration_id,
        }
    }
}

pub fn set_watched_set(
    watched: Vec<(WatchedObjects, SimulationBounds)>,
    watched_objects: &mut HashMap<RigidBodyHandle, WatchedObject>,
    sim_state: &mut SimulationState,
    watch_iteration_id: usize,
) {
    for (watched, region) in watched {
        for (uuid, bsphere) in watched.objects {
            let shape = SharedShape::ball(bsphere.radius);

            let rb_handle = if let Some((rb_handle, body)) = sim_state
                .uuid2body
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
                sim_state.uuid2body.insert(uuid.clone(), rb_handle);
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

        watched.watch_iteration_id == watch_iteration_id
    });
}

pub fn read_watched_objects(
    kvs: &mut KvsContext,
    bounds: SimulationBounds,
) -> Vec<(WatchedObjects, SimulationBounds)> {
    let nbh_regions = bounds.neighbors_to_watch();
    let nbh_watch_keys = nbh_regions.map(|nbh| nbh.watch_kvs_key());

    let mut watched = vec![];
    for (nbh, nbh_key) in nbh_regions.iter().zip(nbh_watch_keys.iter()) {
        if let Ok(data) = kvs.get_with_str_key::<WatchedObjects>(nbh_key) {
            watched.push((data, *nbh));
        }
    }

    watched
}

pub fn compute_watch_data(
    sim_state: &SimulationState,
    watched_objects: &HashMap<RigidBodyHandle, WatchedObject>,
    num_steps_run: usize,
) -> Vec<(Uuid, BoundingSphere)> {
    let mut watch_data = vec![];

    for (handle, body) in sim_state.bodies.iter() {
        if !watched_objects.contains_key(&handle) {
            let uuid = sim_state.body2uuid[&handle].clone();
            let predicted_pos = body.predict_position_using_velocity_and_forces(
                sim_state.params.dt * num_steps_run as f32,
            );
            let aabb = sim_state.colliders[body.colliders()[0]].compute_swept_aabb(&predicted_pos);
            let sphere = BoundingSphere::new(aabb.center(), aabb.half_extents().norm() * 1.1);
            watch_data.push((uuid, sphere));
        }
    }

    watch_data
}
