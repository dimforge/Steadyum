use crate::selection::{SceneMouse, SelectableSceneObject, SelectionShape};
use crate::physics::{PhysicsState, QueryFilter, Ray};

use bevy::prelude::*;

#[cfg(feature = "dim2")]
pub fn update_hovered_entity(
    mut scene_mouse: ResMut<SceneMouse>,
    keyboard: Res<ButtonInput<KeyCode>>,
    physics: Res<PhysicsState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera: Query<(&GlobalTransform, &Camera), With<crate::MainCamera>>,
    gizmo_shapes: Query<(Entity, &GlobalTransform, &SelectionShape)>,
    visibility: Query<&ViewVisibility>,
) {
    use bevy::math::Vec3Swizzles;

    // if !selection_state.inputs_enabled {
    //     return;
    // }

    if !keyboard.pressed(KeyCode::ControlLeft) {
        scene_mouse.hovered = None;
    }

    if let Some(point) = scene_mouse.point {
        // First, check if we are hovering a gizmo.
        let mut gizmo_hit = None;
        for (entity, transform, sel_shape) in gizmo_shapes.iter() {
            if visibility.get(entity).map(|v| v.get()).unwrap_or(false) {
                let shape_shift = Transform {
                    translation: Vec3::new(sel_shape.translation.x, sel_shape.translation.y, 0.0),
                    rotation: Quat::from_rotation_z(sel_shape.rotation.angle()),
                    ..Default::default()
                };
                let total_transform = transform.mul_transform(shape_shift).compute_transform();

                let pos = total_transform.translation.xy();
                let (axis, angle) = total_transform.rotation.to_axis_angle();
                let rot = if axis.z < 0.0 { -angle } else { angle };
                let scale = total_transform.scale.xy();
                let scaled_shape = sel_shape.shape.scale_dyn(scale.into(), 10);

                if let Some(scaled) = scaled_shape {
                    if scaled.contains_point(
                        &crate::physics::rapier::math::Pose::new(pos.into(), rot),
                        point.into(),
                    ) {
                        gizmo_hit = Some(entity);
                    }
                }
            }
        }

        if let Some(entity) = gizmo_hit {
            // Gizmos have priority over everything.
            scene_mouse.hovered = Some(SelectableSceneObject::SelectionShape(entity));
            return;
        }

        // If not, check if we are hovering a scene object.
        // Build filter that only accepts visible colliders.
        let filter = QueryFilter {
            predicate: Some(&|col_handle, _collider| {
                physics
                    .collider_entity(col_handle)
                    .map(|entity| visibility.get(entity).map(|vis| vis.get()).unwrap_or(false))
                    .unwrap_or(false)
            }),
            ..Default::default()
        };
        let qp = physics.broad_phase.as_query_pipeline(
            physics.narrow_phase.query_dispatcher(),
            &physics.bodies,
            &physics.colliders,
            filter,
        );

        let mut topmost_id: u32 = 0;
        for (col_handle, _collider) in qp.intersect_point(point.into()) {
            if let Some(entity) = physics.collider_entity(col_handle) {
                // NOTE: the entities with the largest ids are rendered on top of the ones
                //       with smaller ids (because of the way the debug renderer works).
                //       So we should always select the one with the largest id.
                let eid = entity.index().index();
                if eid >= topmost_id {
                    scene_mouse.hovered = Some(SelectableSceneObject::Collider(entity, point));
                    topmost_id = eid;
                }
            }
        }
    }
}

#[cfg(feature = "dim3")]
pub fn update_hovered_entity(
    mut scene_mouse: ResMut<SceneMouse>,
    physics: Res<PhysicsState>,
    gizmo_shapes: Query<(Entity, &GlobalTransform, &SelectionShape)>,
    visibility: Query<&ViewVisibility>,
) {
    // if !selection_state.inputs_enabled {
    //     return;
    // }

    scene_mouse.hovered = None;

    if let Some((ray_start, ray_dir)) = scene_mouse.ray {
        // First, check if we are hovering a gizmo.
        let mut gizmo_hit = None;
        for (entity, transform, sel_shape) in gizmo_shapes.iter() {
            if visibility.get(entity).map(|v| v.get()).unwrap_or(false) {
                let shape_shift = Transform {
                    translation: sel_shape.translation,
                    rotation: sel_shape.rotation,
                    ..Default::default()
                };
                let total_transform = transform.mul_transform(shape_shift).compute_transform();
                let scale = total_transform.scale;
                let scaled_shape = sel_shape.shape.scale_dyn(scale.into(), 10);

                if let Some(scaled) = scaled_shape {
                    let pose = crate::physics::rapier::math::Pose::from_parts(
                        total_transform.translation,
                        total_transform.rotation,
                    );
                    let ray = Ray::new(ray_start.into(), ray_dir.into());
                    if let Some(toi) = scaled.cast_ray(&pose, &ray, f32::MAX, false) {
                        if toi != f32::MAX {
                            if let Some((best_toi, best_gizmo)) = &mut gizmo_hit {
                                if toi < *best_toi {
                                    *best_toi = toi;
                                    *best_gizmo = entity;
                                }
                            } else {
                                gizmo_hit = Some((toi, entity));
                            }
                        }
                    }
                }
            }
        }

        if let Some((_, entity)) = gizmo_hit {
            // Gizmos have priority over everything.
            scene_mouse.hovered = Some(SelectableSceneObject::SelectionShape(entity));
            return;
        }

        // If not, check if we are hovering a scene object.
        // Build filter that only accepts visible colliders.
        let filter = QueryFilter {
            predicate: Some(&|col_handle, _collider| {
                physics
                    .collider_entity(col_handle)
                    .map(|entity| visibility.get(entity).map(|vis| vis.get()).unwrap_or(false))
                    .unwrap_or(false)
            }),
            ..Default::default()
        };
        let qp = physics.broad_phase.as_query_pipeline(
            physics.narrow_phase.query_dispatcher(),
            &physics.bodies,
            &physics.colliders,
            filter,
        );

        let ray = Ray::new(ray_start, ray_dir);
        let result = qp.cast_ray_and_get_normal(&ray, f32::MAX, true);
        scene_mouse.hovered = result.and_then(|(col_handle, inter)| {
            physics
                .collider_entity(col_handle)
                .map(|entity| SelectableSceneObject::Collider(entity, inter))
        });
    }
}
