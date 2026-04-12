use crate::render::{ColliderRender, ColliderRenderTargets};
use bevy::prelude::*;
use bevy::mesh::{Indices, VertexAttributeValues};

use crate::cli::CliArgs;
use crate::physics::{ColHandle, PhysicsState};
use crate::physics::parry::shape::{Cuboid, TypedShape};

#[derive(Resource, Default, Clone)]
pub struct CollisionShapeMeshInstances {
    cuboid_to_mesh: Vec<(Cuboid, Handle<Mesh>)>, // TODO: make this work for all collider types.
    color_to_material: Vec<(Color, Handle<StandardMaterial>)>,
}

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

#[derive(Component, Copy, Clone)]
pub struct RenderInitialized;

/// System responsible for attaching a PbrBundle to each entity having a collider.
pub fn create_collider_renders_system(
    mut commands: Commands,
    _cli: Res<CliArgs>,
    physics: Res<PhysicsState>,
    mut instances: ResMut<CollisionShapeMeshInstances>,
    mut meshes: ResMut<Assets<Mesh>>,
    #[cfg(feature = "dim2")] mut materials: ResMut<Assets<ColorMaterial>>,
    #[cfg(feature = "dim3")] mut materials: ResMut<Assets<StandardMaterial>>,
    mut coll_shape_render: Query<
        (
            Entity,
            &ColHandle,
            &ColliderRender,
            &mut ColliderRenderTargets,
        ),
        Or<(Changed<ColliderRender>, Without<RenderInitialized>)>,
    >,
    existing_entities: Query<Entity>,
    old_transform: Query<&Transform>,
) {
    for (entity, col_handle, render, mut render_target) in coll_shape_render.iter_mut() {
        let Some(collider) = physics.colliders.get(col_handle.0) else {
            continue;
        };

        commands.entity(entity).insert(RenderInitialized);
        if let Some(mesh) =
            generate_collision_shape_render_mesh(collider, &mut *meshes, &mut instances)
        {
            #[cfg(feature = "dim2")]
            {
                if let TypedShape::Cuboid(s) = collider.shape().as_typed_shape() {
                    let half = s.half_extents;
                    let sprite_bundle = (
                        Sprite {
                            color: render.color.into(),
                            custom_size: Some(Vec2::new(half.x * 2.0, half.y * 2.0)),
                            ..default()
                        },
                        Transform::default(),
                    );

                    if let Some(target) = render_target.target {
                        if existing_entities.get(target).is_ok() {
                            let old_transform = old_transform.get(target).unwrap();
                            commands.entity(target).insert((
                                Sprite {
                                    color: render.color.into(),
                                    custom_size: Some(Vec2::new(half.x * 2.0, half.y * 2.0)),
                                    ..default()
                                },
                                *old_transform,
                            ));
                        }
                    } else {
                        commands.entity(entity).with_children(|cmd| {
                            let target = cmd.spawn(sprite_bundle).id();
                            render_target.target = Some(target);
                        });
                    }
                    continue;
                }
            }

            #[cfg(feature = "dim2")]
            let bundle = (
                Mesh2d(mesh.into()),
                MeshMaterial2d(materials.add(ColorMaterial::from(render.color))),
                Transform::from_xyz(0.0, 0.0, (entity.index().index() + 1) as f32 * 1.0001e-9),
            );

            #[cfg(feature = "dim3")]
            let bundle = {
                let mut material: StandardMaterial = render.color.into();
                material.double_sided = true;

                let material_handle = instances
                    .color_to_material
                    .iter()
                    .find(|(c, _)| *c == render.color)
                    .map(|(_, m)| m.clone())
                    .unwrap_or_else(|| materials.add(material));

                (
                    Mesh3d(mesh),
                    MeshMaterial3d(material_handle),
                    Transform::default(),
                )
            };

            if let Some(target) = render_target.target {
                if existing_entities.get(target).is_ok() {
                    #[cfg(feature = "dim3")]
                    {
                        let old_transform = old_transform.get(target).unwrap();
                        commands.entity(target).insert((
                            Mesh3d(bundle.0 .0.clone()),
                            MeshMaterial3d(bundle.1 .0.clone()),
                            *old_transform,
                        ));
                    }
                    #[cfg(feature = "dim2")]
                    {
                        let old_transform = old_transform.get(target).unwrap();
                        commands.entity(target).insert((
                            Mesh2d(bundle.0 .0.clone()),
                            MeshMaterial2d(bundle.1 .0.clone()),
                            *old_transform,
                        ));
                    }
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
    collider: &crate::physics::rapier::prelude::Collider,
    meshes: &mut Assets<Mesh>,
    instances: &mut CollisionShapeMeshInstances,
) -> Option<Handle<Mesh>> {
    const NSUB: u32 = 20;

    let ((vertices, indices), flat_normals) = match collider.shape().as_typed_shape() {
        TypedShape::Cuboid(s) => {
            if let Some((_, mesh)) = instances
                .cuboid_to_mesh
                .iter()
                .find(|(cuboid, _)| cuboid.half_extents == s.half_extents)
            {
                return Some(mesh.clone());
            }

            let (vertices, indices) = s.to_trimesh();
            let mesh = gen_bevy_mesh(&vertices, &indices, true);
            let handle = meshes.add(mesh);
            instances
                .cuboid_to_mesh
                .push((s.clone(), handle.clone()));
            return Some(handle);
        }
        TypedShape::Ball(s) => (s.to_trimesh(NSUB, NSUB / 2), false),
        TypedShape::Cylinder(s) => {
            let (mut vtx, mut idx) = s.to_trimesh(NSUB);
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
        TypedShape::Cone(s) => {
            let (mut vtx, mut idx) = s.to_trimesh(NSUB);
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
        TypedShape::Capsule(s) => (s.to_trimesh(NSUB, NSUB / 2), false),
        TypedShape::ConvexPolyhedron(s) => (s.to_trimesh(), true),
        TypedShape::HeightField(s) => (s.to_trimesh(), true),
        TypedShape::HalfSpace(s) => {
            let normal: Vec3 = s.normal.into();
            let extent = 100.0;
            let rot = Quat::from_rotation_arc(Vec3::Y, normal);
            let vertices = [
                rot * Vec3::new(extent, 0.0, extent),
                rot * Vec3::new(extent, 0.0, -extent),
                rot * Vec3::new(-extent, 0.0, -extent),
                rot * Vec3::new(-extent, 0.0, extent),
            ];
            let indices = [[0u32, 1, 2], [0, 2, 3]];
            ((vertices.to_vec(), indices.to_vec()), true)
        }
        TypedShape::TriMesh(s) => ((s.vertices().to_vec(), s.indices().to_vec()), true),
        #[cfg(feature = "voxels")]
        TypedShape::Voxels(s) => (s.to_trimesh(), true),
        _ => return None,
    };

    let mesh = gen_bevy_mesh(&vertices, &indices, flat_normals);
    Some(meshes.add(mesh))
}

#[cfg(feature = "dim2")]
fn generate_collision_shape_render_mesh(
    collider: &crate::physics::rapier::prelude::Collider,
    meshes: &mut Assets<Mesh>,
    _unused: &mut CollisionShapeMeshInstances,
) -> Option<Handle<Mesh>> {
    const NSUB: u32 = 20;

    let (vertices, indices) = match collider.shape().as_typed_shape() {
        TypedShape::Cuboid(s) => (s.to_polyline(), None),
        TypedShape::Ball(s) => (s.to_polyline(NSUB), None),
        TypedShape::Capsule(s) => (s.to_polyline(NSUB), None),
        TypedShape::HeightField(_s) => return None,
        TypedShape::TriMesh(s) => (s.vertices().to_vec(), Some(s.indices().to_vec())),
        _ => return None,
    };

    let mesh = gen_bevy_mesh(&vertices, indices);
    Some(meshes.add(mesh))
}

#[cfg(feature = "dim2")]
fn gen_bevy_mesh(vertices: &[Vec2], mut indices: Option<Vec<[u32; 3]>>) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        Default::default(),
    );
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

    mesh.insert_indices(Indices::U32(
        indices
            .unwrap()
            .iter()
            .flat_map(|triangle| triangle.iter())
            .cloned()
            .collect(),
    ));

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
fn gen_bevy_mesh(vertices: &[Vec3], indices: &[[u32; 3]], flat_normals: bool) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        Default::default(),
    );
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
            let normal = ab.cross(ac);
            // Contribute this normal to each vertex in the triangle.
            for i in 0..3 {
                normals[triangle[i] as usize] += normal;
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
    mesh.insert_indices(Indices::U32(
        indices
            .iter()
            .flat_map(|triangle| triangle.iter())
            .cloned()
            .collect(),
    ));

    if flat_normals {
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
    }

    mesh
}
