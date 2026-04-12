use bevy::prelude::*;
use super::state::PhysicsState;
use super::systems;

/// Plugin that sets up physics simulation using rapier directly.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsState>()
            .add_systems(Update, (
                systems::physics_step,
                systems::sync_physics_to_transforms.after(systems::physics_step),
            ));
    }
}
