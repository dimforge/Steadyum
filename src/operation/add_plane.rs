use crate::operation::{Operation, Operations};
use crate::physics::{ColHandle, ColliderBuilder, PhysicsState, RbHandle, RigidBodyBuilder, RigidBodyType};
use crate::physics::rapier::na::Unit;
use crate::render::ColliderRender;
use bevy::prelude::*;

pub fn add_plane(
    mut commands: Commands,
    operations: Res<Operations>,
    mut physics: ResMut<PhysicsState>,
) {
    for op in operations.iter() {
        if let Operation::AddPlane = op {
            let entity = commands
                .spawn((
                    ColliderRender::default(),
                    Transform::default(),
                    Visibility::default(),
                ))
                .id();

            // Create a fixed rigid body in PhysicsState.
            let rb_handle = physics.insert_body(
                entity,
                RigidBodyBuilder::new(RigidBodyType::Fixed),
            );
            commands.entity(entity).insert(RbHandle(rb_handle));

            // Create a halfspace collider (ground plane with normal pointing up).
            #[cfg(feature = "dim2")]
            let collider = ColliderBuilder::halfspace(Unit::new_unchecked(Vec2::Y));
            #[cfg(feature = "dim3")]
            let collider = ColliderBuilder::halfspace(Unit::new_unchecked(Vec3::Y));
            let col_handle = physics.insert_collider_with_parent(entity, collider, rb_handle);
            commands.entity(entity).insert(ColHandle(col_handle));
        }
    }
}
