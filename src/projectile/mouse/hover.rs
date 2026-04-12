use crate::selection::SceneMouse;
use bevy::prelude::*;

#[cfg(feature = "dim3")]
use {crate::drag::DragState, crate::physics::parry::query};

#[cfg(feature = "dim2")]
pub fn handle_projectile_hover(mut _commands: Commands) {}

#[cfg(feature = "dim3")]
pub fn handle_projectile_hover(
    drag_state: ResMut<DragState>,
    scene_mouse: Res<SceneMouse>,
    mut physics: ResMut<crate::physics::PhysicsState>,
) {
    if let Some((ray_orig, ray_dir)) = scene_mouse.ray {
        if let Some(entity) = drag_state.mouse_body {
            if let Some(body_handle) = physics.body_handle(entity) {
                // Cast the ray on the plane.
                if let Some(toi) = query::details::line_toi_with_halfspace(
                    drag_state.drag_plane_point,
                    drag_state.drag_plane_normal,
                    ray_orig,
                    ray_dir,
                ) {
                    let new_pos = ray_orig + ray_dir * toi;
                    if let Some(body) = physics.bodies.get_mut(body_handle) {
                        body.set_translation(new_pos, true);
                    }
                }
            }
        }
    }
}
