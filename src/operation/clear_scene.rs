use crate::operation::{Operation, Operations, PersistentIntersection};
use crate::PhysicsProgress;
use bevy::prelude::*;
use bevy_rapier::prelude::*;

pub fn clear_scene(
    mut commands: Commands,
    mut progress: ResMut<PhysicsProgress>,
    operations: Res<Operations>,
    to_remove: Query<
        Entity,
        Or<(
            With<RapierRigidBodyHandle>,
            With<RapierColliderHandle>,
            With<PersistentIntersection>,
        )>,
    >,
) {
    for op in operations.iter() {
        if let Operation::ClearScene = op {
            progress.simulated_time = 0.0;
            for entity in to_remove.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
