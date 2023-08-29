use crate::render::{ColliderRender, ColliderRenderTargets};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy_rapier::geometry::{Collider, ColliderView};
use bevy_rapier::rapier::math::{Point, Real, Vector};
use na::{point, UnitQuaternion};

use crate::cli::CliArgs;
use crate::storage::External;
#[cfg(feature = "dim2")]
use bevy::sprite::MaterialMesh2dBundle;

pub fn add_collider_render_targets(
    mut commands: Commands,
    missing_targets: Query<Entity, (With<ColliderRender>, Without<ColliderRenderTargets>)>,
) {
    for entity in missing_targets.iter() {
        commands
            .entity(entity)
            .insert(ColliderRenderTargets::default());
    }
}

/// System responsible for attaching a PbrBundle to each entity having a collider.
pub fn create_collider_renders_system(
    mut commands: Commands,
    cli: Res<CliArgs>,
    mut meshes: ResMut<Assets<Mesh>>,
    #[cfg(feature = "dim2")] mut materials: ResMut<Assets<ColorMaterial>>,
    #[cfg(feature = "dim3")] mut materials: ResMut<Assets<StandardMaterial>>,
    mut coll_shape_render: Query<
        (
            Entity,
            &Collider,
            &ColliderRender,
            &mut ColliderRenderTargets,
        ),
        Or<(Changed<Collider>, Changed<ColliderRender>)>,
    >,
    mut external_coll_shape_render: Query<
        (
            Entity,
            &External<Collider>,
            &ColliderRender,
            &mut ColliderRenderTargets,
        ),
        (
            Or<(Changed<External<Collider>>, Changed<ColliderRender>)>,
            Without<Collider>,
        ),
    >,
    existing_entities: Query<Entity>,
    old_transform: Query<&Transform>, // FIXME: we shouldn’t need this, but right now collider renders get triggered each frame for some reasons.
) {
    for (entity, collider, render, mut render_target) in coll_shape_render.iter_mut().chain(
        external_coll_shape_render
            .iter_mut()
            .map(|(e, c, cr, trgt)| (e, &c.0, cr, trgt)),
    ) {
        if let Some(mesh) =
            generate_collision_shape_render_mesh(cli.lower_graphics, collider, &mut *meshes)
        {
            let mut material: StandardMaterial = render.color.into();
            material.double_sided = true;

            // println!("Rendering with color: {:?}", render.color);

            #[cfg(feature = "dim2")]
            {
                if let ColliderView::Cuboid(s) = collider.as_unscaled_typed_shape() {
                    #[cfg(feature = "dim2")]
                    let mut bundle = SpriteBundle {
                        sprite: Sprite {
                            color: render.color.into(),
                            custom_size: Some(Vec2::new(
                                s.half_extents().x * 2.0,
                                s.half_extents().y * 2.0,
                            )),
                            ..default()
                        },
                        ..default()
                    };

                    if let Some(target) = render_target.target {
                        if existing_entities.get(target).is_ok() {
                            let old_transform = old_transform.get(target).unwrap();
                            bundle.transform = *old_transform;
                            commands.entity(target).insert(bundle);
                        }
                    } else {
                        commands.entity(entity).with_children(|cmd| {
                            let target = cmd.spawn(bundle).id();
                            render_target.target = Some(target);
                        });
                    }
                    continue;
                }
            }

            #[cfg(feature = "dim2")]
            let mut bundle = MaterialMesh2dBundle {
                mesh: mesh.into(),
                material: materials.add(render.color.into()),
                transform: Transform::from_xyz(0.0, 0.0, (entity.index() + 1) as f32 * 1.0001e-9),
                ..Default::default()
            };

            #[cfg(feature = "dim3")]
            let mut bundle = PbrBundle {
                mesh,
                material: materials.add(material),
                ..Default::default()
            };

            if let Some(target) = render_target.target {
                if existing_entities.get(target).is_ok() {
                    let old_transform = old_transform.get(target).unwrap();
                    bundle.transform = *old_transform;
                    commands.entity(target).insert(bundle);
                }
            } else {
                commands.entity(entity).with_children(|cmd| {
                    let target = cmd.spawn(bundle).id();
                    render_target.target = Some(target);
                });
            }
        }
    }
}

#[cfg(feature = "dim3")]
fn generate_collision_shape_render_mesh(
    lower_graphics: bool,
    collider: &Collider,
    meshes: &mut Assets<Mesh>,
) -> Option<Handle<Mesh>> {
    const NSUB: u32 = 20;

    let ((vertices, indices), flat_normals) = match collider.as_unscaled_typed_shape() {
        ColliderView::Cuboid(s) => {
            if lower_graphics {
                // INSTANCED
                return None;
            } else {
                (s.raw.to_trimesh(), true)
            }
        }
        ColliderView::Ball(s) => (s.raw.to_trimesh(NSUB, NSUB / 2), false),
        ColliderView::Cylinder(s) => {
            let (mut vtx, mut idx) = s.raw.to_trimesh(NSUB);
            // Duplicate the basis of the cylinder, to get nice normals.
            let base_id = vtx.len() as u32;

            for i in 0..vtx.len() {
                vtx.push(vtx[i]);
            }

            for idx in &mut idx[NSUB as usize * 2..] {
                idx[0] += base_id;
                idx[1] += base_id;
                idx[2] += base_id;
            }

            ((vtx, idx), false)
        }
        ColliderView::Cone(s) => {
            let (mut vtx, mut idx) = s.raw.to_trimesh(NSUB);
            // Duplicate the basis of the cone, to get nice normals.
            let base_id = vtx.len() as u32;

            for i in 0..vtx.len() - 1 {
                vtx.push(vtx[i]);
            }

            for idx in &mut idx[NSUB as usize..] {
                idx[0] += base_id;
                idx[1] += base_id;
                idx[2] += base_id;
            }

            ((vtx, idx), false)
        }
        ColliderView::Capsule(s) => (s.raw.to_trimesh(NSUB, NSUB / 2), false),
        ColliderView::ConvexPolyhedron(s) => (s.raw.to_trimesh(), true),
        // ColliderView::Compound(s) => s.raw.to_trimesh(),
        ColliderView::HeightField(s) => (s.raw.to_trimesh(), true),
        // ColliderView::Polyline(s) => s.raw.to_trimesh(),
        // ColliderView::Triangle(s) => s.raw.to_trimesh(),
        ColliderView::HalfSpace(s) => {
            let normal = s.normal();
            let extent = 100.0;
            let rot = UnitQuaternion::rotation_between(&Vector::y(), &normal.into())
                .unwrap_or(UnitQuaternion::identity());
            let vertices = [
                rot * point![extent, 0.0, extent],
                rot * point![extent, 0.0, -extent],
                rot * point![-extent, 0.0, -extent],
                rot * point![-extent, 0.0, extent],
            ];
            let indices = [[0, 1, 2], [0, 2, 3]];
            ((vertices.to_vec(), indices.to_vec()), true)
        }
        ColliderView::TriMesh(s) => ((s.raw.vertices().to_vec(), s.indices().to_vec()), true),
        #[cfg(feature = "voxels")]
        ColliderView::Voxels(s) => (s.raw.to_trimesh(), true),
        _ => todo!(),
    };

    let mesh = gen_bevy_mesh(&vertices, &indices, flat_normals);
    Some(meshes.add(mesh))
}

#[cfg(feature = "dim2")]
fn generate_collision_shape_render_mesh(
    lower_graphics: bool,
    collider: &Collider,
    meshes: &mut Assets<Mesh>,
) -> Option<Handle<Mesh>> {
    const NSUB: u32 = 20;

    let (vertices, indices) = match collider.as_unscaled_typed_shape() {
        ColliderView::Cuboid(s) => {
            if lower_graphics {
                // INSTANCED
                // return None;
                (s.raw.to_polyline(), None)
            } else {
                (s.raw.to_polyline(), None)
            }
        }
        ColliderView::Ball(s) => (s.raw.to_polyline(NSUB), None),
        ColliderView::Capsule(s) => (s.raw.to_polyline(NSUB), None),
        // ColliderView::ConvexPolygon(s) => (s.raw.to_polyline(), None),
        // ColliderView::Compound(s) => s.raw.to_polyline(),
        ColliderView::HeightField(s) => return None, // (s.raw.to_polyline(), None),
        // ColliderView::Polyline(s) => s.raw.to_polyline(),
        // ColliderView::Triangle(s) => s.raw.to_polyline(),
        ColliderView::TriMesh(s) => (s.raw.vertices().to_vec(), Some(s.indices().to_vec())),
        _ => todo!(),
    };

    let mesh = gen_bevy_mesh(&vertices, indices);
    Some(meshes.add(mesh))
}

#[cfg(feature = "dim2")]
fn gen_bevy_mesh(vertices: &[Point<Real>], mut indices: Option<Vec<[u32; 3]>>) -> Mesh {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(
            vertices
                .iter()
                .map(|vertex| [vertex.x, vertex.y, 0.0])
                .collect::<Vec<_>>(),
        ),
    );

    if indices.is_none() {
        indices = Some(
            (1..vertices.len() as u32 - 1)
                .map(|i| [0, i, i + 1])
                .collect(),
        );
    }

    mesh.set_indices(Some(Indices::U32(
        indices
            .unwrap()
            .iter()
            .flat_map(|triangle| triangle.iter())
            .cloned()
            .collect(),
    )));

    let normals: Vec<_> = (0..vertices.len()).map(|_| [0.0, 0.0, 1.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::from(normals));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::from(
            (0..vertices.len())
                .map(|_vertex| [0.0, 0.0])
                .collect::<Vec<_>>(),
        ),
    );

    mesh
}

#[cfg(feature = "dim3")]
fn gen_bevy_mesh(vertices: &[Point<Real>], indices: &[[u32; 3]], flat_normals: bool) -> Mesh {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(
            vertices
                .iter()
                .map(|vertex| [vertex.x, vertex.y, vertex.z])
                .collect::<Vec<_>>(),
        ),
    );

    if !flat_normals {
        // Compute vertex normals by averaging the normals
        // of every triangle they appear in.
        // NOTE: This is a bit shonky, but good enough for visualisation.
        let mut normals: Vec<Vec3> = vec![Vec3::ZERO; vertices.len()];
        for triangle in indices.iter() {
            let ab = vertices[triangle[1] as usize] - vertices[triangle[0] as usize];
            let ac = vertices[triangle[2] as usize] - vertices[triangle[0] as usize];
            let normal = ab.cross(&ac);
            // Contribute this normal to each vertex in the triangle.
            for i in 0..3 {
                normals[triangle[i] as usize] += Vec3::new(normal.x, normal.y, normal.z);
            }
        }

        let normals: Vec<[f32; 3]> = normals
            .iter()
            .map(|normal| {
                let normal = normal.normalize();
                [normal.x, normal.y, normal.z]
            })
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::from(normals));
    }

    // There's nothing particularly meaningful we can do
    // for this one without knowing anything about the overall topology.
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::from(
            vertices
                .iter()
                .map(|_vertex| [0.0, 0.0])
                .collect::<Vec<_>>(),
        ),
    );
    mesh.set_indices(Some(Indices::U32(
        indices
            .iter()
            .flat_map(|triangle| triangle.iter())
            .cloned()
            .collect(),
    )));

    if flat_normals {
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
    }

    mesh
}
