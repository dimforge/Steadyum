use crate::operation::{Operation, Operations};
use crate::render::ColliderRender;
use crate::utils::RigidBodyBundle;
use bevy::prelude::*;
use bevy_rapier::prelude::*;

pub fn add_plane(mut commands: Commands, operations: Res<Operations>) {
    for op in operations.iter() {
        if let Operation::AddPlane = op {
            commands
                .spawn(Collider::halfspace(Vect::Y).unwrap())
                .insert_bundle(RigidBodyBundle::fixed())
                .insert(ColliderRender::default());
        }
    }
}
