use crate::selection::SceneMouse;
use bevy::prelude::*;

#[cfg(feature = "dim3")]
use {crate::drag::DragState, bevy_rapier::parry::query};

#[cfg(feature = "dim2")]
pub fn handle_drag_hover(mut commands: Commands) {}

#[cfg(feature = "dim3")]
pub fn handle_drag_hover(
    drag_state: ResMut<DragState>,
    scene_mouse: Res<SceneMouse>,
    mut transforms: Query<&mut Transform>,
) {
    if let Some((ray_orig, ray_dir)) = scene_mouse.ray {
        if let Some(mut mouse_body_transform) = drag_state
            .mouse_body
            .and_then(|e| transforms.get_mut(e).ok())
        {
            // Cast the ray on the plane.
            if let Some(toi) = query::details::line_toi_with_halfspace(
                &drag_state.drag_plane_point.into(),
                &drag_state.drag_plane_normal.into(),
                &ray_orig.into(),
                &ray_dir.into(),
            ) {
                mouse_body_transform.translation = ray_orig + ray_dir * toi;
            }
        }
    }
}
