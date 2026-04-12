use crate::insertion::{InsertionPreview, InsertionState, InsertionStep};
use crate::selection::SceneMouse;
use crate::ui::ActiveMouseAction;
use bevy::prelude::*;
use crate::physics::{PhysicsState, QueryFilter};

use crate::styling::Theme;
#[cfg(feature = "dim3")]
use crate::physics::parry::query::details;

#[cfg(feature = "dim2")]
pub fn update_preview_scale(
    mut commands: Commands,
    mut insertion_state: ResMut<InsertionState>,
    mut mouse_action: ResMut<ActiveMouseAction>,
    physics: Res<PhysicsState>,
    theme: Res<Theme>,
    scene_mouse: Res<SceneMouse>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    preview: Query<(Entity, &InsertionPreview)>,
) {
    use bevy::math::Vec3Swizzles;
    let (preview_entity, _) = preview.single().unwrap();

    if let Some(point) = scene_mouse.point {
        match insertion_state.step {
            Some(InsertionStep::Basis) => {
                if (insertion_state.start_point - insertion_state.end_point).length()
                    >= crate::insertion::ACTIVE_EPS
                {
                    *mouse_action = ActiveMouseAction::Insertion;
                }

                insertion_state.end_point = point;

                let mut transform = insertion_state.transform();
                // NOTE: in 2D we have to rebuild the preview shape,
                //       otherwise the borders will be scaled.
                let new_preview = crate::insertion::preview_shape_bundle(
                    transform.scale.xy(),
                    theme.insertion_preview_color(),
                );
                transform.scale = Vec3::ONE;

                commands
                    .entity(preview_entity)
                    .insert(new_preview)
                    .insert(transform);
            }
            _ => {}
        }
    }
}

#[cfg(feature = "dim3")]
pub fn update_preview_scale(
    mut commands: Commands,
    mut insertion_state: ResMut<InsertionState>,
    mut mouse_action: ResMut<ActiveMouseAction>,
    physics: Res<PhysicsState>,
    theme: Res<Theme>,
    scene_mouse: Res<SceneMouse>,
    mut gizmos: Gizmos,
    preview: Query<(Entity, &InsertionPreview)>,
) {
    let (preview_entity, _) = preview.single().unwrap();

    if let Some((ray_orig, ray_dir)) = scene_mouse.ray {
        match insertion_state.step {
            Some(InsertionStep::Basis) => {
                if let Some(hit) = details::line_toi_with_halfspace(
                    insertion_state.start_point,
                    insertion_state.normal(),
                    ray_orig,
                    ray_dir,
                ) {
                    if (insertion_state.start_point - insertion_state.end_point).length()
                        >= crate::insertion::ACTIVE_EPS
                    {
                        *mouse_action = ActiveMouseAction::Insertion;
                    }

                    insertion_state.end_point = ray_orig + ray_dir * hit;
                    insertion_state.height = 1.0e-6;

                    commands
                        .entity(preview_entity)
                        .insert(insertion_state.transform());
                }
            }
            Some(InsertionStep::Height) => {
                let (height, _) = details::closest_points_line_line_parameters(
                    insertion_state.end_point,
                    insertion_state.normal(),
                    ray_orig,
                    ray_dir,
                );
                insertion_state.height = height;

                commands
                    .entity(preview_entity)
                    .insert(insertion_state.transform());
            }
            _ => {}
        }

        // Check intersection with environment to color the preview.
        let transform = insertion_state.transform();
        let shift = insertion_state.normal() * 1.0e-3;
        let preview_shape_clone = insertion_state.preview_shape.clone();
        if let Some(preview_shape) = &preview_shape_clone {
            // Scale the preview shape to match the visible preview size.
            let scaled_preview = preview_shape.scale_dyn(transform.scale, 20);

            let query_pipeline = physics.broad_phase.as_query_pipeline(
                physics.narrow_phase.query_dispatcher(),
                &physics.bodies,
                &physics.colliders,
                QueryFilter::default(),
            );

            use crate::physics::rapier::math::Pose;
            let pose = Pose::from_parts(transform.translation + shift, transform.rotation);
            let shape_ref: &dyn crate::physics::parry::shape::Shape = scaled_preview
                .as_deref()
                .unwrap_or_else(|| &**preview_shape);
            let mut inter = query_pipeline.intersect_shape(pose, shape_ref);

            if inter.next().is_some() {
                insertion_state.intersects_environment = true;
            } else {
                insertion_state.intersects_environment = false;
            }
        }

        // Draw wireframe preview using gizmos.
        if insertion_state.step.is_some() {
            let transform = insertion_state.transform();
            let polyline = crate::styling::cuboid_polyline();
            let color = if insertion_state.intersects_environment {
                Color::srgb(1.0, 0.0, 0.0)
            } else {
                theme.insertion_preview_color()
            };

            for pair in polyline.vertices.windows(2) {
                let a = transform.transform_point(pair[0]);
                let b = transform.transform_point(pair[1]);
                gizmos.line(a, b, color);
            }
        }
    }
}
