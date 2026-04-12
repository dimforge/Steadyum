use crate::selection::transform_gizmo::GizmoTransformable;
use crate::physics::ColHandle;
use bevy::prelude::*;

pub fn add_missing_gizmos(
    mut commands: Commands,
    colliders_wo_picking: Query<(Entity, &ColHandle), Without<GizmoTransformable>>,
) {
    for (entity, _col_handle) in colliders_wo_picking.iter() {
        commands
            .entity(entity)
            .insert(GizmoTransformable);
    }
}
