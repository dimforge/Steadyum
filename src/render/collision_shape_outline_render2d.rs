use crate::render::{ColliderOutlineRender, ColliderRenderTargets};
use crate::CliArgs;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::physics::{ColHandle, PhysicsState};
use crate::physics::parry::shape::TypedShape;
use bevy_prototype_lyon::entity::Shape;

pub fn create_collider_outline_renders_system(
    mut commands: Commands,
    cli: Res<CliArgs>,
    physics: Res<PhysicsState>,
    mut coll_shape_render: Query<
        (
            Entity,
            &ColHandle,
            &mut ColliderOutlineRender,
            &mut ColliderRenderTargets,
        ),
        Or<(Changed<ColHandle>, Changed<ColliderOutlineRender>)>,
    >,
    existing_entities: Query<Entity>,
) {
    if cli.lower_graphics {
        return;
    }

    for (entity, col_handle, mut render, mut targets) in coll_shape_render.iter_mut() {
        let Some(collider) = physics.colliders.get(col_handle.0) else {
            continue;
        };

        if let Some((shape, z_transform)) = generate_collision_shape_render_outline(
            entity,
            collider,
            render.color,
            render.thickness,
        ) {
            if let Some(target) = targets.outline_target {
                if existing_entities.get(target).is_ok() {
                    commands.entity(target).insert(shape).insert(z_transform);
                }
            } else {
                commands.entity(entity).with_children(|cmd| {
                    let target = cmd.spawn((shape, z_transform)).id();
                    targets.outline_target = Some(target);
                });
            }
        }
    }
}

fn generate_collision_shape_render_outline(
    entity: Entity,
    collider: &crate::physics::rapier::prelude::Collider,
    color: Color,
    thickness: f32,
) -> Option<(Shape, Transform)> {
    const NSUB: u32 = 20;

    let (vertices, _indices, closed) = match collider.shape().as_typed_shape() {
        TypedShape::Cuboid(s) => (s.to_polyline(), None::<Vec<[u32; 3]>>, true),
        TypedShape::Ball(s) => (s.to_polyline(NSUB), None, true),
        TypedShape::Capsule(s) => (s.to_polyline(NSUB), None, true),
        TypedShape::HeightField(s) => {
            let (vtx, _) = s.to_polyline();
            // FIXME: set the indices too
            (vtx, None, false)
        }
        TypedShape::TriMesh(s) => (s.vertices().to_vec(), Some(s.indices().to_vec()), true),
        _ => return None,
    };

    let polygon = bevy_prototype_lyon::shapes::Polygon {
        points: vertices
            .iter()
            .map(|pt| Vec2::new(pt.x, pt.y))
            .collect(),
        closed,
    };

    let shape = ShapeBuilder::with(&polygon)
        .stroke(Stroke::new(color, thickness))
        .build();

    let z = (entity.index().index() + 1) as f32 * 1.0e-9;
    Some((shape, Transform::from_xyz(0.0, 0.0, z)))
}
