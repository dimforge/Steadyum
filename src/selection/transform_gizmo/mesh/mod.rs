use crate::layers::GIZMO_LAYER;
use crate::selection::transform_gizmo::{
    gizmo_material::{GizmoMaterial, GizmoStateMaterials},
    PickableGizmo, TransformGizmoBundle, TransformGizmoInteraction,
};
use crate::selection::SelectionShape;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_rapier::geometry::Collider;

mod arrow;
mod truncated_torus;

/// Startup system that builds the procedural mesh and materials of the gizmo.
#[cfg(feature = "dim2")]
pub fn build_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let axis_length = 0.4;
    let arc_radius = 0.1;

    let arrow = arrow::Arrow {
        radius: 0.02,
        length: axis_length,
        head_radius: 0.08,
        head_length: 0.2,
    };
    let translation_mesh = meshes.add(Mesh::from(arrow));
    let translation_selection = SelectionShape::new(Collider::cuboid(
        arrow.head_radius,
        arrow.head_radius + arrow.head_length * 2.0,
    ));

    let ring_radius = 0.015;
    let rotation_mesh = meshes.add(Mesh::from(truncated_torus::TruncatedTorus {
        radius: arc_radius,
        ring_radius,
        ..Default::default()
    }));
    let rotation_selection = SelectionShape {
        translation: Vec2::ZERO,
        rotation: 0.0,
        shape: Collider::ball(arc_radius),
    };

    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.1 }));

    // Define gizmo materials
    let (s, l, a) = (0.45, 0.59, 1.0);
    let gizmo_matl_x = materials.add(ColorMaterial::from(Color::hsla(351.0, s, l, a)));
    let gizmo_matl_y = materials.add(ColorMaterial::from(Color::hsla(82.0, s, l, a)));
    let gizmo_matl_z = materials.add(ColorMaterial::from(Color::hsla(212.0, s, l, a)));

    let (s, l, a) = (1.0, 0.59, 1.0);
    let gizmo_matl_x_sel = materials.add(ColorMaterial::from(Color::hsla(351.0, s, l, a)));
    let gizmo_matl_y_sel = materials.add(ColorMaterial::from(Color::hsla(82.0, s, l, a)));
    let gizmo_matl_z_sel = materials.add(ColorMaterial::from(Color::hsla(212.0, s, l, a)));

    let matls_x = GizmoStateMaterials {
        idle: gizmo_matl_x.clone(),
        hovered: gizmo_matl_x_sel.clone(),
    };
    let matls_y = GizmoStateMaterials {
        idle: gizmo_matl_y.clone(),
        hovered: gizmo_matl_y_sel.clone(),
    };
    let matls_z = GizmoStateMaterials {
        idle: gizmo_matl_z.clone(),
        hovered: gizmo_matl_z_sel.clone(),
    };

    // Build the gizmo using the variables above.
    commands
        .spawn_bundle(TransformGizmoBundle::default())
        .with_children(|parent| {
            // Translation Handles
            parent
                .spawn_bundle(ColorMesh2dBundle {
                    mesh: translation_mesh.clone().into(),
                    material: gizmo_matl_x.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                        Vec3::new(axis_length / 2.0 + arc_radius, 0.0, 0.0),
                    )),
                    ..Default::default()
                })
                .insert(matls_x.clone())
                .insert(translation_selection.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                });
            parent
                .spawn_bundle(ColorMesh2dBundle {
                    mesh: translation_mesh.clone().into(),
                    material: gizmo_matl_y.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        axis_length / 2.0 + arc_radius,
                        0.0,
                    )),
                    ..Default::default()
                })
                .insert(matls_y.clone())
                .insert(translation_selection.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                });

            // Rotation Arcs
            parent
                .spawn_bundle(ColorMesh2dBundle {
                    mesh: rotation_mesh.clone().into(),
                    material: gizmo_matl_z.clone(),
                    ..Default::default()
                })
                .insert(matls_z.clone())
                .insert(rotation_selection.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::RotateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                });

            // Scaling Handles
            parent
                .spawn_bundle(ColorMesh2dBundle {
                    mesh: cube_mesh.clone().into(),
                    material: gizmo_matl_x.clone(),
                    transform: Transform::from_translation(Vec3::new(axis_length + 0.5, 0.0, 0.0)),
                    ..Default::default()
                })
                .insert(matls_x.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::ScaleAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                });
            parent
                .spawn_bundle(ColorMesh2dBundle {
                    mesh: cube_mesh.clone().into(),
                    material: gizmo_matl_y.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, axis_length + 0.5, 0.0)),
                    ..Default::default()
                })
                .insert(matls_y.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::ScaleAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                });
        });
}

/// Startup system that builds the procedural mesh and materials of the gizmo.
#[cfg(feature = "dim3")]
pub fn build_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GizmoMaterial>>,
) {
    let axis_length = 0.6;
    let arc_radius = 0.3;

    let arrow = arrow::Arrow {
        radius: 0.02,
        length: axis_length,
        head_radius: 0.08,
        head_length: 0.2,
    };
    let translation_mesh = meshes.add(Mesh::from(arrow));
    let translation_selection =
        SelectionShape::new(Collider::capsule_y(axis_length / 2.0, arrow.head_radius));

    // let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere {
    //     radius: 0.02,
    //     subdivisions: 3,
    // }));
    let ring_radius = 0.015;
    let rotation_mesh = meshes.add(Mesh::from(truncated_torus::TruncatedTorus {
        radius: arc_radius,
        ring_radius,
        ..Default::default()
    }));

    let rotation_selection = SelectionShape {
        translation: Vec3::new(arc_radius / 1.5, 0.0, arc_radius / 1.5),
        rotation: Quat::IDENTITY,
        shape: Collider::cuboid(arc_radius / 1.5, ring_radius * 2.0, arc_radius / 1.5),
    };

    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.1 }));

    // Define gizmo materials
    let (s, l, a) = (0.45, 0.59, 1.0);
    let gizmo_matl_x = materials.add(GizmoMaterial::from(Color::hsla(351.0, s, l, a)));
    let gizmo_matl_y = materials.add(GizmoMaterial::from(Color::hsla(82.0, s, l, a)));
    let gizmo_matl_z = materials.add(GizmoMaterial::from(Color::hsla(212.0, s, l, a)));

    let (s, l, a) = (1.0, 0.59, 1.0);
    let gizmo_matl_x_sel = materials.add(GizmoMaterial::from(Color::hsla(351.0, s, l, a)));
    let gizmo_matl_y_sel = materials.add(GizmoMaterial::from(Color::hsla(82.0, s, l, a)));
    let gizmo_matl_z_sel = materials.add(GizmoMaterial::from(Color::hsla(212.0, s, l, a)));

    let matls_x = GizmoStateMaterials {
        idle: gizmo_matl_x.clone(),
        hovered: gizmo_matl_x_sel.clone(),
    };
    let matls_y = GizmoStateMaterials {
        idle: gizmo_matl_y.clone(),
        hovered: gizmo_matl_y_sel.clone(),
    };
    let matls_z = GizmoStateMaterials {
        idle: gizmo_matl_z.clone(),
        hovered: gizmo_matl_z_sel.clone(),
    };

    /*let gizmo_matl_origin = materials.add(StandardMaterial {
        unlit: true,
        base_color: Color::rgb(0.7, 0.7, 0.7),
        ..Default::default()
    });*/
    // Build the gizmo using the variables above.
    commands
        .spawn_bundle(TransformGizmoBundle::default())
        .with_children(|parent| {
            // Translation Handles
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: translation_mesh.clone(),
                    material: gizmo_matl_x.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                        Vec3::new(axis_length, 0.0, 0.0),
                    )),
                    ..Default::default()
                })
                .insert(matls_x.clone())
                .insert(translation_selection.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: translation_mesh.clone(),
                    material: gizmo_matl_y.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, axis_length, 0.0)),
                    ..Default::default()
                })
                .insert(matls_y.clone())
                .insert(translation_selection.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: translation_mesh.clone(),
                    material: gizmo_matl_z.clone(),
                    transform: Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, 0.0, axis_length),
                    )),
                    ..Default::default()
                })
                .insert(matls_z.clone())
                .insert(translation_selection)
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::TranslateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));
            // Rotation Arcs
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: rotation_mesh.clone(),
                    material: gizmo_matl_x.clone(),
                    transform: Transform::from_rotation(Quat::from_axis_angle(
                        Vec3::Z,
                        f32::to_radians(90.0),
                    )),
                    ..Default::default()
                })
                .insert(rotation_selection.clone())
                .insert(matls_x.clone())
                .insert(TransformGizmoInteraction::RotateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: rotation_mesh.clone(),
                    material: gizmo_matl_y.clone(),
                    ..Default::default()
                })
                .insert(rotation_selection.clone())
                .insert(matls_y.clone())
                .insert(TransformGizmoInteraction::RotateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: rotation_mesh.clone(),
                    material: gizmo_matl_z.clone(),
                    transform: Transform::from_rotation(
                        Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))
                            * Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0)),
                    ),
                    ..Default::default()
                })
                .insert(rotation_selection.clone())
                .insert(matls_z.clone())
                .insert(TransformGizmoInteraction::RotateAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));

            // Scaling Handles
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: cube_mesh.clone(),
                    material: gizmo_matl_x.clone(),
                    transform: Transform::from_translation(Vec3::new(axis_length + 0.5, 0.0, 0.0)),
                    ..Default::default()
                })
                .insert(matls_x.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::ScaleAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: cube_mesh.clone(),
                    material: gizmo_matl_y.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, axis_length + 0.5, 0.0)),
                    ..Default::default()
                })
                .insert(matls_y.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::ScaleAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));
            parent
                .spawn_bundle(MaterialMeshBundle {
                    mesh: cube_mesh.clone(),
                    material: gizmo_matl_z.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, axis_length + 0.5)),
                    ..Default::default()
                })
                .insert(matls_z.clone())
                .insert(PickableGizmo::default())
                .insert(TransformGizmoInteraction::ScaleAxis {
                    original: Vec3::Z,
                    axis: Vec3::Z,
                })
                .insert(NotShadowCaster)
                .insert(RenderLayers::layer(GIZMO_LAYER));
        });
}
