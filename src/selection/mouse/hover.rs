use crate::selection::{SceneMouse, SelectableSceneObject, SelectionShape};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier::prelude::*;

#[cfg(feature = "dim2")]
pub fn update_hovered_entity(
    mut scene_mouse: ResMut<SceneMouse>,
    keyboard: Res<Input<KeyCode>>,
    physics: Res<RapierContext>,
    windows: Query<&Window, With<PrimaryWindow>>,
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
                    rotation: Quat::from_rotation_z(sel_shape.rotation),
                    ..Default::default()
                };
                let total_transform = transform.mul_transform(shape_shift).compute_transform();
                let mut scaled_shape = sel_shape.shape.clone();
                scaled_shape.set_scale(total_transform.scale.xy(), 10);

                if scaled_shape.contains_point(
                    total_transform.translation.xy(),
                    total_transform.rotation.to_axis_angle().1,
                    point,
                ) {
                    gizmo_hit = Some(entity);
                }
            }
        }

        if let Some(entity) = gizmo_hit {
            // Gizmos have priority over everything.
            scene_mouse.hovered = Some(SelectableSceneObject::SelectionShape(entity));
            return;
        }

        // If not, check if we are hovering a scene object.
        let mut topmost_id = 0;
        physics.intersections_with_point(
            point,
            QueryFilter::default()
                .predicate(&|entity| visibility.get(entity).map(|vis| vis.get()).unwrap_or(false)),
            |entity| {
                // NOTE: the entities with the largest ids are rendered on top of the ones
                //       with smaller ids (because of the way the bevy_rapier debug renderer works).
                //       So we should always select the one with the largest id.
                if entity.index() >= topmost_id {
                    scene_mouse.hovered = Some(SelectableSceneObject::Collider(entity, point));
                    topmost_id = entity.index();
                }
                true
            },
        );
    }
}

#[cfg(feature = "dim3")]
pub fn update_hovered_entity(
    mut scene_mouse: ResMut<SceneMouse>,
    physics: Res<RapierContext>,
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
                let mut scaled_shape = sel_shape.shape.clone();
                scaled_shape.set_scale(total_transform.scale, 10);

                if let Some(toi) = scaled_shape.cast_ray(
                    total_transform.translation,
                    total_transform.rotation,
                    ray_start,
                    ray_dir,
                    f32::MAX,
                    false,
                ) {
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

        if let Some((_, entity)) = gizmo_hit {
            // Gizmos have priority over everything.
            scene_mouse.hovered = Some(SelectableSceneObject::SelectionShape(entity));
            return;
        }

        // If not, check if we are hovering a scene object.
        scene_mouse.hovered = physics
            .cast_ray_and_get_normal(
                ray_start,
                ray_dir,
                f32::MAX,
                false,
                QueryFilter::default().predicate(&|entity| {
                    visibility.get(entity).map(|vis| vis.get()).unwrap_or(false)
                }),
            )
            .map(|(entity, inter)| SelectableSceneObject::Collider(entity, inter));
    }
}
