use crate::render::{ColliderOutlineRender, ColliderRender, ColliderRenderTargets};
use crate::CliArgs;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier::plugin::RapierConfiguration;
use bevy_rapier::prelude::{Collider, ColliderView};
use bevy_rapier::rapier::math::{Point, Real, Vector};

pub fn create_collider_outline_renders_system(
    mut commands: Commands,
    cli: Res<CliArgs>,
    mut coll_shape_render: Query<
        (
            Entity,
            &Collider,
            &mut ColliderOutlineRender,
            &mut ColliderRenderTargets,
        ),
        Or<(Changed<Collider>, Changed<ColliderOutlineRender>)>,
    >,
    existing_entities: Query<Entity>,
) {
    if cli.lower_graphics {
        return;
    }

    for (entity, collider, mut render, mut targets) in coll_shape_render.iter_mut() {
        if let Some(bundle) = generate_collision_shape_render_outline(
            entity,
            collider,
            render.color,
            render.thickness,
        ) {
            if let Some(target) = targets.outline_target {
                if existing_entities.get(target).is_ok() {
                    commands.entity(target).insert(bundle);
                }
            } else {
                commands.entity(entity).with_children(|cmd| {
                    let target = cmd.spawn(bundle).id();
                    targets.outline_target = Some(target);
                });
            }
        }
    }
}

fn generate_collision_shape_render_outline(
    entity: Entity,
    collider: &Collider,
    color: Color,
    thickness: f32,
) -> Option<ShapeBundle> {
    const NSUB: u32 = 20;

    let (vertices, indices, closed) = match collider.as_unscaled_typed_shape() {
        ColliderView::Cuboid(s) => (s.raw.to_polyline(), None, true),
        ColliderView::Ball(s) => (s.raw.to_polyline(NSUB), None, true),
        ColliderView::Capsule(s) => (s.raw.to_polyline(NSUB), None, true),
        // ColliderView::ConvexPolygon(s) => (s.raw.to_polyline(), None),
        // ColliderView::Compound(s) => s.raw.to_polyline(),
        ColliderView::HeightField(s) => {
            let (vtx, _) = s.raw.to_polyline();
            // FIXME: set the indices too
            (vtx, None, false)
        }
        // ColliderView::Polyline(s) => s.raw.to_polyline(),
        // ColliderView::Triangle(s) => s.raw.to_polyline(),
        ColliderView::TriMesh(s) => (s.raw.vertices().to_vec(), Some(s.indices().to_vec()), true),
        _ => todo!(),
    };

    let polygon = bevy_prototype_lyon::shapes::Polygon {
        points: vertices
            .iter()
            .map(|pt| Vec2::from(*pt) * collider.scale())
            .collect(),
        closed,
    };

    Some(GeometryBuilder::build_as(
        &polygon,
        DrawMode::Stroke(StrokeMode::new(color, thickness)),
        Transform::from_xyz(0.0, 0.0, (entity.index() + 1) as f32 * 1.0e-9)
            * Transform::from_scale(Vec3::new(
                // The scale is already taken into account in the polygon geometry.
                1.0 / collider.scale().x,
                1.0 / collider.scale().y,
                1.0,
            )),
    ))
}
