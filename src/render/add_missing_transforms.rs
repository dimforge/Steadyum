use crate::render::{ColliderOutlineRender, ColliderRender};
use bevy::prelude::*;
use crate::physics::{PhysicsState, pose_to_transform};

/// Automatically adds the Transform and GlobalTransform components to entities with a
/// debug-renderer component. Without these, transform propagation won't reach the
/// rendered meshes.
pub fn add_missing_transforms(
    mut commands: Commands,
    physics: Res<PhysicsState>,
    renderable: Query<
        (Entity, Option<&Transform>, Option<&GlobalTransform>),
        (
            Or<(With<ColliderRender>, With<ColliderOutlineRender>)>,
            Or<(Without<Transform>, Without<GlobalTransform>)>,
        ),
    >,
) {
    for (entity, transform, global_transform) in renderable.iter() {
        if transform.is_none() {
            if let Some(collider) = physics
                .entity2collider
                .get(&entity)
                .and_then(|h| physics.colliders.get(*h))
            {
                commands
                    .entity(entity)
                    .insert(pose_to_transform(&collider.position()));
            }
        }
        if global_transform.is_none() {
            commands.entity(entity).insert(GlobalTransform::default());
        }
    }
}
