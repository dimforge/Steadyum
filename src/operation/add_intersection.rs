use crate::operation::Operations;
use bevy::prelude::*;
use bevy_rapier::prelude::*;

#[cfg(feature = "dim3")]
use {
    crate::operation::Operation, crate::utils, bevy::pbr::wireframe::Wireframe,
    bevy_rapier::parry::query::SplitResult, bevy_rapier::rapier::math::Isometry,
};

#[derive(Component)]
pub struct PersistentIntersection(Entity, Entity);

#[cfg(feature = "dim2")] // TODO: not implemented in 2D yet.
pub fn add_intersection(
    mut _commands: Commands,
    _operations: Res<Operations>,
    _colliders: Query<(Entity, &Collider)>,
) {
}

#[cfg(feature = "dim2")] // TODO: not implemented in 2D yet.
pub fn update_intersection(
    mut _commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    _intersections: Query<(Entity, &PersistentIntersection)>,
    _transforms: Query<&Transform, Changed<Transform>>,
    _global_transforms: Query<(&GlobalTransform, Changed<GlobalTransform>)>,
    _shapes: Query<&Collider>,
) {
}

#[cfg(feature = "dim3")]
pub fn add_intersection(
    mut commands: Commands,
    operations: Res<Operations>,
    colliders: Query<(Entity, &Collider)>,
) {
    for op in operations.iter() {
        if let Operation::AddIntersection = op {
            // FIXME: this is just a very specialized version to test
            // the plane/mesh splitting.
            let mut trimesh = None;
            let mut plane = None;

            for (entity, collider) in colliders.iter() {
                let shape = collider.as_typed_shape();
                if matches!(shape, ColliderView::TriMesh { .. }) {
                    if trimesh.is_some() {
                        plane = Some(entity);
                    } else {
                        trimesh = Some(entity);
                    }
                }

                if matches!(shape, ColliderView::HalfSpace { .. })
                    || matches!(shape, ColliderView::Cuboid { .. })
                {
                    plane = Some(entity);
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
    global_transforms: Query<(&GlobalTransform, Changed<GlobalTransform>)>,
    shapes: Query<&Collider>,
) {
    for (entity, intersection) in intersections.iter() {
        if let (Ok((t1, changed1)), Ok((t2, changed2))) = (
            global_transforms.get(intersection.0),
            global_transforms.get(intersection.1),
        ) {
            let t1 = t1.compute_transform();
            let t2 = t2.compute_transform();

            if changed1 || changed2 {
                if let (Ok(shape1), Ok(shape2)) =
                    (shapes.get(intersection.0), shapes.get(intersection.1))
                {
                    // Compute the intersection.
                    let mesh1 = shape1.as_trimesh().unwrap();
                    let mesh1_pos: Isometry<f32> = (t1.translation, t1.rotation).into();

                    if let Some(mesh2) = shape2.as_trimesh() {
                        let mesh2_pos: Isometry<f32> = (t2.translation, t2.rotation).into();

                        match crate::parry::transformation::intersect_meshes(
                            &mesh1_pos, &mesh1.raw, false, &mesh2_pos, &mesh2.raw, false,
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
                                commands.entity(entity).remove::<PbrBundle>();
                            }
                            Err(err) => error!("mesh intersection failed {}", err),
                        }
                    }

                    if let Some(plane) = shape2.as_halfspace() {
                        let plane_pos: Isometry<f32> = (t2.translation, t2.rotation).into();
                        let axis = plane_pos * plane.raw.normal;
                        let bias = plane_pos.translation.vector.dot(&axis);

                        match mesh1.raw.split(&mesh1_pos, &axis, bias, 1.0e-5) {
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
                                    &mesh1.raw,
                                    mesh1_pos,
                                );
                                commands.entity(entity).insert(bundle).insert(Wireframe);
                            }
                            _ => {
                                commands.entity(entity).remove::<PbrBundle>();
                            }
                        };
                    }

                    if let Some(cuboid) = shape2.as_cuboid() {
                        let cuboid_pos: Isometry<f32> = (t2.translation, t2.rotation).into();

                        if let Some(intersection) = mesh1.raw.intersection_with_cuboid(
                            &mesh1_pos,
                            false,
                            &cuboid.raw,
                            &cuboid_pos,
                            false,
                            1.0e-5,
                        ) {
                            let bundle = utils::bevy_pbr_bundle_from_trimesh(
                                &mut meshes,
                                &intersection,
                                mesh1_pos,
                            );
                            commands.entity(entity).insert(bundle).insert(Wireframe);
                        }
                    }
                }
            }
        }
    }
}
