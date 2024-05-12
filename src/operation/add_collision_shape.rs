use crate::operation::{Operation, Operations};
use crate::styling::ColorGenerator;
use crate::utils::ColliderRenderBundle;
use bevy::prelude::*;

pub fn add_collision_shape(
    mut commands: Commands,
    operations: Res<Operations>,
    mut colors: ResMut<ColorGenerator>,
) {
    for op in operations.iter() {
        if let Operation::AddCollider(collider, rigid_body, transform) = op {
            commands
                .spawn(collider.clone())
                .insert(rigid_body.clone())
                .insert(TransformBundle::from_transform(*transform))
                .insert(ColliderRenderBundle::new(&mut colors));
        }
    }
}
