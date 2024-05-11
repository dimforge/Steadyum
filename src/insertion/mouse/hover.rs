use crate::insertion::{InsertionPreview, InsertionState, InsertionStep};
use crate::selection::SceneMouse;
use crate::ui::ActiveMouseAction;
use bevy::prelude::*;
use bevy_polyline::material::PolylineMaterial;
use bevy_rapier::prelude::{QueryFilter, RapierContext};

use crate::styling::Theme;
#[cfg(feature = "dim3")]
use bevy_rapier::parry::query::details;

#[cfg(feature = "dim2")]
pub fn update_preview_scale(
    mut commands: Commands,
    mut insertion_state: ResMut<InsertionState>,
    mut mouse_action: ResMut<ActiveMouseAction>,
    physics: Res<RapierContext>,
    theme: Res<Theme>,
    scene_mouse: Res<SceneMouse>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<PolylineMaterial>>,
    preview: Query<(Entity, &InsertionPreview)>,
) {
    use bevy::math::Vec3Swizzles;
    let (preview_entity, _) = preview.get_single().unwrap();

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
    physics: Res<RapierContext>,
    theme: Res<Theme>,
    scene_mouse: Res<SceneMouse>,
    mut materials: ResMut<Assets<PolylineMaterial>>,
    preview: Query<(Entity, &InsertionPreview, &Handle<PolylineMaterial>)>,
) {
    let (preview_entity, _, mat_handle) = preview.get_single().unwrap();

    if let Some((ray_orig, ray_dir)) = scene_mouse.ray {
        match insertion_state.step {
            Some(InsertionStep::Basis) => {
                if let Some(hit) = details::line_toi_with_halfspace(
                    &insertion_state.start_point.into(),
                    &insertion_state.normal().into(),
                    &ray_orig.into(),
                    &ray_dir.into(),
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
                    &insertion_state.end_point.into(),
                    &insertion_state.normal().into(),
                    &ray_orig.into(),
                    &ray_dir.into(),
                );
                insertion_state.height = height;

                commands
                    .entity(preview_entity)
                    .insert(insertion_state.transform());
            }
            _ => {}
        }

        // Color the preview in red if it intersects another shapes.
        let transform = insertion_state.transform();
        let shift = insertion_state.normal() * 1.0e-3;
        if let Some(preview_shape) = &mut insertion_state.preview_shape {
            preview_shape.set_scale(transform.scale, 10);
            let inter = physics.intersection_with_shape(
                // We slightly offset the shape so it doesn’t intersect
                // with the flat plane it lies on.
                transform.translation + shift,
                transform.rotation,
                preview_shape,
                QueryFilter::default(),
            );

            if inter.is_some() {
                insertion_state.intersects_environment = true;
                materials.get_mut(mat_handle).unwrap().color = Color::RED;
            } else {
                insertion_state.intersects_environment = false;
                materials.get_mut(mat_handle).unwrap().color = theme.insertion_preview_color();
            }
        }
    }
}
