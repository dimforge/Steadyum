use crate::render::{ColliderOutlineRender, ColliderRender};
use bevy::prelude::*;
use bevy_rapier::plugin::RapierContext;

/// Automatically adds the Transform and GlobalTransform components to entities with a
/// debug-renderer component. Without these, transform propagation won’t reach the
/// rendered meshes.
pub fn add_missing_transforms(
    mut commands: Commands,
    context: ResMut<RapierContext>,
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
            if let Some(collider) = context
                .entity2collider()
                .get(&entity)
                .and_then(|h| context.colliders.get(*h))
            {
                commands
                    .entity(entity)
                    .insert(bevy_rapier::utils::iso_to_transform(collider.position()));
            }
        }
        if global_transform.is_none() {
            commands.entity(entity).insert(GlobalTransform::default());
        }
    }
}
