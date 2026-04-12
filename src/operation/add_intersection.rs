use crate::operation::Operations;
use crate::physics::{ColHandle, PhysicsState, Pose};
use bevy::prelude::*;

#[cfg(feature = "dim3")]
use {
    crate::operation::Operation,
    crate::utils,
    bevy::pbr::wireframe::Wireframe,
    crate::physics::parry::query::SplitResult,
    crate::physics::parry::shape::TypedShape,
};

#[derive(Component)]
pub struct PersistentIntersection(Entity, Entity);

#[cfg(feature = "dim2")] // TODO: not implemented in 2D yet.
pub fn add_intersection(
    mut _commands: Commands,
    _operations: Res<Operations>,
    _col_entities: Query<Entity, With<ColHandle>>,
    _physics: Res<PhysicsState>,
) {
}

#[cfg(feature = "dim2")] // TODO: not implemented in 2D yet.
pub fn update_intersection(
    mut _commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    _intersections: Query<(Entity, &PersistentIntersection)>,
    _transforms: Query<&Transform, Changed<Transform>>,
    _global_transforms: Query<&GlobalTransform>,
    _physics: Res<PhysicsState>,
) {
}

#[cfg(feature = "dim3")]
pub fn add_intersection(
    mut commands: Commands,
    operations: Res<Operations>,
    col_entities: Query<Entity, With<ColHandle>>,
    physics: Res<PhysicsState>,
) {
    for op in operations.iter() {
        if let Operation::AddIntersection = op {
            // FIXME: this is just a very specialized version to test
            // the plane/mesh splitting.
            let mut trimesh = None;
            let mut plane = None;

            for entity in col_entities.iter() {
                if let Some(col_handle) = physics.collider_handle(entity) {
                    if let Some(collider) = physics.colliders.get(col_handle) {
                        let shape = collider.shared_shape();
                        match shape.as_typed_shape() {
                            TypedShape::TriMesh(_) => {
                                if trimesh.is_some() {
                                    plane = Some(entity);
                                } else {
                                    trimesh = Some(entity);
                                }
                            }
                            TypedShape::HalfSpace(_) | TypedShape::Cuboid(_) => {
                                plane = Some(entity);
                            }
                            _ => {}
                        }
                    }
                }
            }

            if let (Some(trimesh), Some(plane)) = (trimesh, plane) {
                commands.spawn(PersistentIntersection(trimesh, plane));
            }
        }
    }
}

#[cfg(feature = "dim3")]
pub fn update_intersection(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    intersections: Query<(Entity, &PersistentIntersection)>,
    global_transforms: Query<Ref<GlobalTransform>>,
    physics: Res<PhysicsState>,
) {
    for (entity, intersection) in intersections.iter() {
        if let (Ok(t1), Ok(t2)) = (
            global_transforms.get(intersection.0),
            global_transforms.get(intersection.1),
        ) {
            let t1_changed = t1.is_changed();
            let t2_changed = t2.is_changed();
            let t1 = t1.compute_transform();
            let t2 = t2.compute_transform();

            if t1_changed || t2_changed {
                let col1 = physics
                    .collider_handle(intersection.0)
                    .and_then(|h| physics.colliders.get(h));
                let col2 = physics
                    .collider_handle(intersection.1)
                    .and_then(|h| physics.colliders.get(h));

                if let (Some(shape1), Some(shape2)) = (col1, col2) {
                    let shape1 = shape1.shared_shape();
                    let shape2 = shape2.shared_shape();

                    // Compute the intersection.
                    let mesh1_pos = Pose::from_parts(t1.translation, t1.rotation.into());

                    if let Some(mesh1_trimesh) = shape1.as_trimesh() {
                        if let Some(mesh2_trimesh) = shape2.as_trimesh() {
                            let mesh2_pos =
                                Pose::from_parts(t2.translation, t2.rotation.into());

                            match crate::physics::parry::transformation::intersect_meshes(
                                &mesh1_pos,
                                mesh1_trimesh,
                                false,
                                &mesh2_pos,
                                mesh2_trimesh,
                                false,
                            ) {
                                Ok(Some(result)) => {
                                    let bundle = utils::bevy_pbr_bundle_from_trimesh(
                                        &mut meshes,
                                        &result,
                                        mesh1_pos,
                                    );
                                    commands.entity(entity).insert(bundle).insert(Wireframe);
                                }
                                Ok(None) => {
                                    commands
                                        .entity(entity)
                                        .remove::<(Mesh3d, Transform)>();
                                }
                                Err(err) => error!("mesh intersection failed {}", err),
                            }
                        }

                        if let Some(halfspace) = shape2.as_halfspace() {
                            let plane_pos =
                                Pose::from_parts(t2.translation, t2.rotation.into());
                            let axis = plane_pos.rotation * halfspace.normal;
                            let bias = plane_pos.translation.dot(axis);

                            match mesh1_trimesh.split(&mesh1_pos, axis, bias, 1.0e-5) {
                                SplitResult::Pair(piece, _) => {
                                    let bundle = utils::bevy_pbr_bundle_from_trimesh(
                                        &mut meshes,
                                        &piece,
                                        mesh1_pos,
                                    );
                                    commands.entity(entity).insert(bundle).insert(Wireframe);
                                }
                                SplitResult::Negative => {
                                    let bundle = utils::bevy_pbr_bundle_from_trimesh(
                                        &mut meshes,
                                        mesh1_trimesh,
                                        mesh1_pos,
                                    );
                                    commands.entity(entity).insert(bundle).insert(Wireframe);
                                }
                                _ => {
                                    commands
                                        .entity(entity)
                                        .remove::<(Mesh3d, Transform)>();
                                }
                            };
                        }

                        if let Some(cuboid) = shape2.as_cuboid() {
                            let cuboid_pos =
                                Pose::from_parts(t2.translation, t2.rotation.into());

                            match mesh1_trimesh.intersection_with_cuboid(
                                &mesh1_pos,
                                false,
                                cuboid,
                                &cuboid_pos,
                                false,
                                1.0e-5,
                            ) {
                                Ok(Some(intersection_result)) => {
                                    let bundle = utils::bevy_pbr_bundle_from_trimesh(
                                        &mut meshes,
                                        &intersection_result,
                                        mesh1_pos,
                                    );
                                    commands.entity(entity).insert(bundle).insert(Wireframe);
                                }
                                Ok(None) => {
                                    commands
                                        .entity(entity)
                                        .remove::<(Mesh3d, Transform)>();
                                }
                                Err(err) => error!("cuboid intersection failed {}", err),
                            }
                        }
                    }
                }
            }
        }
    }
}
