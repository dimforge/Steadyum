use crate::operation::{Operation, Operations, PersistentIntersection};
use crate::physics::{ColHandle, PhysicsState, RbHandle};
use crate::PhysicsProgress;
use bevy::prelude::*;

pub fn clear_scene(
    mut commands: Commands,
    mut progress: ResMut<PhysicsProgress>,
    operations: Res<Operations>,
    mut physics: ResMut<PhysicsState>,
    to_remove: Query<
        Entity,
        Or<(
            With<RbHandle>,
            With<ColHandle>,
            With<PersistentIntersection>,
        )>,
    >,
) {
    for op in operations.iter() {
        if let Operation::ClearScene = op {
            progress.simulated_time = 0.0;
            for entity in to_remove.iter() {
                commands.entity(entity).despawn();
            }
            // Clear all rapier state (bodies, colliders, joints, mappings).
            physics.clear();
        }
    }
}
