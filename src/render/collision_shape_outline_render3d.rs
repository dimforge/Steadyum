use crate::cli::CliArgs;
use crate::render::ColliderOutlineRender;
use bevy::prelude::*;
use crate::physics::{ColHandle, PhysicsState};
use crate::physics::parry::shape::TypedShape;

/// System that draws collider outlines using Bevy gizmos each frame.
/// Replaces the old bevy_polyline-based approach.
pub fn create_collider_outline_renders_system(
    cli: Res<CliArgs>,
    physics: Res<PhysicsState>,
    mut gizmos: Gizmos,
    coll_shape_render: Query<
        (
            Entity,
            &ColHandle,
            &ColliderOutlineRender,
        ),
    >,
) {
    if cli.lower_graphics {
        return;
    }

    for (_entity, col_handle, render) in coll_shape_render.iter() {
        let Some(collider) = physics.colliders.get(col_handle.0) else {
            continue;
        };

        if let Some((vertices, indices)) = generate_collision_shape_outline(collider) {
            let pos = collider.position();
            let translation = pos.translation;
            let rotation: Quat = pos.rotation.into();

            for idx in &indices {
                let a = translation + rotation * vertices[idx[0] as usize];
                let b = translation + rotation * vertices[idx[1] as usize];
                gizmos.line(a, b, render.color);
            }
        }
    }
}

fn generate_collision_shape_outline(
    collider: &crate::physics::rapier::prelude::Collider,
) -> Option<(Vec<Vec3>, Vec<[u32; 2]>)> {
    const NSUB: u32 = 20;

    let (vertices, indices) = match collider.shape().as_typed_shape() {
        TypedShape::Cuboid(s) => s.to_outline(),
        TypedShape::Ball(s) => s.to_outline(NSUB),
        TypedShape::Cylinder(s) => s.to_outline(NSUB),
        TypedShape::Cone(s) => s.to_outline(NSUB),
        TypedShape::Capsule(s) => s.to_outline(NSUB),
        #[cfg(feature = "voxels")]
        TypedShape::Voxels(s) => s.to_outline(),
        TypedShape::ConvexPolyhedron(_s) => return None, // TODO
        TypedShape::HeightField(_s) => return None,
        TypedShape::HalfSpace(_s) => return None, // TODO
        TypedShape::TriMesh(_s) => return None,
        _ => return None,
    };

    Some((vertices, indices))
}
