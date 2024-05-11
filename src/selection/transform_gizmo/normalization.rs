use crate::MainCamera;
use bevy::transform::TransformSystem;
use bevy::{prelude::*, render::camera::Camera};

use super::{GizmoSettings, TransformGizmoSystem};

pub struct Ui3dNormalization;
impl Plugin for Ui3dNormalization {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            normalize
                .in_set(TransformGizmoSystem::NormalizeSet)
                .after(TransformSystem::TransformPropagate)
                .after(TransformGizmoSystem::Place)
                .run_if(|settings: Res<GizmoSettings>| settings.enabled),
        );
    }
}

/// Marker struct that marks entities with meshes that should be scaled relative to the camera.
#[derive(Component)]
pub struct Normalize3d {
    /// Length of the object in world space units
    pub size_in_world: f32,
    /// Desired length of the object in pixels
    pub desired_pixel_size: f32,
}

impl Normalize3d {
    pub fn new(size_in_world: f32, desired_pixel_size: f32) -> Self {
        Normalize3d {
            size_in_world,
            desired_pixel_size,
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn normalize(
    mut query: ParamSet<(
        Query<(&GlobalTransform, &Camera), With<MainCamera>>,
        Query<(&mut Transform, &mut GlobalTransform, &Normalize3d)>,
    )>,
) {
    if cfg!(feature = "dim3") {
        // TODO: can be improved by manually specifying the active camera to normalize against. The
        // majority of cases will only use a single camera for this viewer, so this is sufficient.
        let p0 = query.p0();
        let (camera_position, camera) = p0.get_single().expect("Not exactly one camera");
        let camera_position = camera_position.to_owned();
        let camera = camera.to_owned();
        let view = camera_position.compute_matrix().inverse();

        for (mut transform, mut global_transform, normalize) in query.p1().iter_mut() {
            let decomposed_global_transform = global_transform.compute_transform();
            let distance = view
                .transform_point3(decomposed_global_transform.translation)
                .z;

            let pixel_end = if let Some(coords) = camera.world_to_viewport(
                &GlobalTransform::default(),
                Vec3::new(
                    normalize.size_in_world * decomposed_global_transform.scale.x,
                    0.0,
                    distance,
                ),
            ) {
                coords
            } else {
                continue;
            };

            let pixel_root = if let Some(coords) =
                camera.world_to_viewport(&GlobalTransform::default(), Vec3::new(0.0, 0.0, distance))
            {
                coords
            } else {
                continue;
            };

            let actual_pixel_size = pixel_root.distance(pixel_end);
            let required_scale = normalize.desired_pixel_size / actual_pixel_size;

            transform.scale = decomposed_global_transform.scale * Vec3::splat(required_scale);
            *global_transform = (*transform).into();
        }
    }
}
