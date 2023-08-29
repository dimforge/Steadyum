use crate::selection::SceneMouse;

use crate::MainCamera;
use bevy::prelude::*;

pub fn track_mouse_state(
    windows: Res<Windows>,
    mut scene_mouse: ResMut<SceneMouse>,
    camera: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
) {
    if let Some(window) = windows.get_primary() {
        for (camera_transform, camera) in camera.iter() {
            if let Some(cursor) = window.cursor_position() {
                let ndc_cursor =
                    (cursor / Vec2::new(window.width(), window.height()) * 2.0) - Vec2::ONE;
                let ndc_to_world =
                    camera_transform.compute_matrix() * camera.projection_matrix().inverse();
                let ray_pt1 =
                    ndc_to_world.project_point3(Vec3::new(ndc_cursor.x, ndc_cursor.y, -1.0));

                #[cfg(feature = "dim2")]
                {
                    use bevy::math::Vec3Swizzles;
                    scene_mouse.point = Some(ray_pt1.xy());
                }
                #[cfg(feature = "dim3")]
                {
                    let ray_pt2 =
                        ndc_to_world.project_point3(Vec3::new(ndc_cursor.x, ndc_cursor.y, 1.0));
                    let ray_dir = ray_pt2 - ray_pt1;
                    scene_mouse.ray = Some((ray_pt1, ray_dir));
                }
            }
        }
    }
}
