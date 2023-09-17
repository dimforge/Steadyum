use crate::cli::CliArgs;
use crate::render::{ColliderOutlineRender, ColliderRenderTargets};
use bevy::prelude::*;
use bevy_polyline::prelude::*;
use bevy_rapier::prelude::{Collider, ColliderView};
use bevy_rapier::rapier::math::{Point, Real};

pub fn create_collider_outline_renders_system(
    mut commands: Commands,
    cli: Res<CliArgs>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut materials: ResMut<Assets<PolylineMaterial>>,
    mut coll_shape_render: Query<
        (
            Entity,
            &Collider,
            &ColliderOutlineRender,
            &mut ColliderRenderTargets,
        ),
        Or<(Changed<Collider>, Changed<ColliderOutlineRender>)>,
    >,
    existing_entities: Query<Entity>,
    old_transform: Query<&Transform>,
) {
    if cli.lower_graphics {
        return;
    }

    for (entity, collider, render, mut render_target) in coll_shape_render.iter_mut() {
        if let Some(polyline) = generate_collision_shape_render_outline(collider) {
            let material = PolylineMaterial {
                color: render.color,
                width: render.thickness,
                perspective: true,
                ..Default::default()
            };

            let mut bundle = PolylineBundle {
                polyline: polylines.add(polyline),
                material: materials.add(material),
                ..Default::default()
            };

            if let Some(target) = render_target.outline_target {
                if existing_entities.get(target).is_ok() {
                    let old_transform = old_transform.get(target).unwrap();
                    bundle.transform = *old_transform;
                    commands.entity(target).insert(bundle);
                }
            } else {
                let target = commands.entity(entity).with_children(|cmd| {
                    let target = cmd.spawn(bundle).id();
                    render_target.outline_target = Some(target);
                });
            }
        }
    }
}

fn generate_collision_shape_render_outline(collider: &Collider) -> Option<Polyline> {
    const NSUB: u32 = 20;

    let (vertices, indices) = match collider.as_unscaled_typed_shape() {
        ColliderView::Cuboid(s) => s.raw.to_outline(),
        ColliderView::Ball(s) => s.raw.to_outline(NSUB),
        ColliderView::Cylinder(s) => s.raw.to_outline(NSUB),
        ColliderView::Cone(s) => s.raw.to_outline(NSUB),
        ColliderView::Capsule(s) => s.raw.to_outline(NSUB),
        #[cfg(feature = "voxels")]
        ColliderView::Voxels(s) => s.raw.to_outline(),
        ColliderView::ConvexPolyhedron(_s) => todo!(),
        // ColliderView::Compound(s) => s.raw.to_trimesh(),
        ColliderView::HeightField(_s) => return None,
        // ColliderView::Polyline(s) => s.raw.to_trimesh(),
        // ColliderView::Triangle(s) => s.raw.to_trimesh(),
        ColliderView::HalfSpace(_s) => {
            todo!()
        }
        ColliderView::TriMesh(_s) => return None,
        _ => todo!(),
    };

    Some(gen_bevy_polyline(&vertices, &indices))
}

fn gen_bevy_polyline(pts: &[Point<Real>], indices: &[[u32; 2]]) -> Polyline {
    let mut vertices = vec![];
    let mut last_id = indices[0][0];

    for idx in indices {
        if last_id == idx[0] {
            vertices.push(pts[idx[0] as usize].into());
        } else {
            // Break the polyline by inserting an invalid point.
            vertices.push(Vec3::splat(f32::NAN));
            vertices.push(pts[idx[0] as usize].into());
        }
        vertices.push(pts[idx[1] as usize].into());
        last_id = idx[1];
    }

    Polyline { vertices }
}
