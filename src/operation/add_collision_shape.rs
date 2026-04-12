use crate::operation::{Operation, Operations};
use crate::physics::{PhysicsState, ColHandle, ColliderBuilder, RbHandle, SharedShape, transform_to_pose};
use std::sync::Arc;
use crate::styling::ColorGenerator;
use crate::utils::ColliderRenderBundle;
use bevy::prelude::*;

pub fn add_collision_shape(
    mut commands: Commands,
    operations: Res<Operations>,
    mut colors: ResMut<ColorGenerator>,
    mut physics: ResMut<PhysicsState>,
) {
    for op in operations.iter() {
        if let Operation::AddCollider(collider_bundle, rigid_body_bundle, transform) = op {
            // Spawn the Bevy entity with position/rotation but scale=1
            // (the scale is baked into the rapier collider shape).
            let entity = commands
                .spawn((
                    Name::new("Collision Shape"),
                    Transform {
                        translation: transform.translation,
                        rotation: transform.rotation,
                        scale: Vec3::ONE,
                    },
                    Visibility::default(),
                ))
                .id();

            // Insert render components.
            let render_bundle = ColliderRenderBundle::new(&mut colors);
            commands.entity(entity).insert((
                render_bundle.render,
                render_bundle.render_outline,
            ));

            // Create the rapier rigid body and collider in PhysicsState,
            // applying the entity transform as the body's position.
            let pose = transform_to_pose(transform);
            let rb_builder = rigid_body_bundle
                .into_builder()
                .position(pose);
            let rb_handle = physics.insert_body(entity, rb_builder);
            commands.entity(entity).insert(RbHandle(rb_handle));

            // Scale the collider shape to match the entity's transform scale.
            // The preview shape is unit-sized; the actual size comes from transform.scale.
            #[cfg(feature = "dim3")]
            let scale = transform.scale;
            #[cfg(feature = "dim2")]
            let scale = transform.scale.truncate();
            let scaled_collider = if scale != crate::physics::Vect::ONE {
                if let Some(scaled_shape) = collider_bundle.collider.shared_shape().scale_dyn(
                    scale,
                    20, // num subdivisions for curved shapes
                ) {
                    ColliderBuilder::new(SharedShape(Arc::from(scaled_shape))).build()
                } else {
                    collider_bundle.collider.clone()
                }
            } else {
                collider_bundle.collider.clone()
            };

            let col_handle = physics.insert_collider_with_parent(
                entity,
                scaled_collider,
                rb_handle,
            );
            commands.entity(entity).insert(ColHandle(col_handle));
        }
    }
}
