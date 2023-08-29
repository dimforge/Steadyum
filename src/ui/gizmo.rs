use crate::selection::transform_gizmo::GizmoTransformable;
use bevy::prelude::*;
use bevy_rapier::prelude::*;

pub fn add_missing_gizmos(
    mut commands: Commands,
    colliders_wo_picking: Query<(Entity, &Collider), Without<GizmoTransformable>>,
) {
    for (entity, _collider) in colliders_wo_picking.iter() {
        commands
            .entity(entity)
            // .insert_bundle(bevy_mod_picking::PickableBundle::default())
            .insert(GizmoTransformable);
    }
}
