use crate::MainCamera;
use bevy::prelude::*;
use crate::physics::{PhysicsState, ColHandle, RbHandle, SharedShape, Vect, QueryFilter, Pose};

pub struct ControlPlugin;

/// Our own character controller component, replacing bevy_rapier's
/// KinematicCharacterController / KinematicCharacterControllerOutput.
#[derive(Clone, Debug, Component)]
pub struct CharacterController {
    /// The collider shape used for the character sweep.
    pub shape: SharedShape,
    /// Desired translation for this frame (set by control_characters).
    pub translation: Option<Vect>,
    /// Whether the character is on the ground.
    pub grounded: bool,
    /// The effective translation that was actually applied last frame.
    pub effective_translation: Vect,
    /// The desired translation that was requested last frame.
    pub desired_translation: Vect,
    /// Character controller offset (skin width).
    pub offset: f32,
    /// Max slope angle the character can climb (radians).
    pub max_slope_climb_angle: f32,
    /// Min slope angle that causes the character to slide.
    pub min_slope_slide_angle: f32,
    /// Snap to ground distance.
    pub snap_to_ground: Option<f32>,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            shape: SharedShape::ball(0.5),
            translation: None,
            grounded: false,
            effective_translation: Vect::ZERO,
            desired_translation: Vect::ZERO,
            offset: 0.01,
            max_slope_climb_angle: std::f32::consts::FRAC_PI_4,
            min_slope_slide_angle: std::f32::consts::FRAC_PI_4,
            snap_to_ground: Some(0.2),
        }
    }
}

#[derive(Copy, Clone, Debug, Component)]
pub struct CharacterControlOptions {
    pub enabled: bool,
    pub velocity: Vect,
    pub gravity_scale: f32,
}

impl Default for CharacterControlOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            velocity: Vect::ZERO,
            gravity_scale: 8.0,
        }
    }
}

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, control_characters)
            .add_systems(Update, apply_character_movements);
    }
}

/// Compute desired translations for each character controller.
pub fn control_characters(
    events: Res<ButtonInput<KeyCode>>,
    physics: Res<PhysicsState>,
    mut characters: Query<(
        &mut CharacterController,
        &mut CharacterControlOptions,
    )>,
    cameras: Query<&GlobalTransform, With<MainCamera>>,
) {
    if !physics.running {
        return;
    }

    if let Ok(camera_transform) = cameras.single() {
        for (mut character, mut options) in characters.iter_mut() {
            let options = &mut *options;
            if !options.enabled {
                continue;
            }

            let dt = physics.integration_parameters.dt;
            let inv_dt = physics.integration_parameters.inv_dt();
            let gravity_vel = physics.gravity * dt * options.gravity_scale;

            options.velocity = Vect::ZERO;
            options.velocity.y = character
                .effective_translation
                .y
                .min(character.desired_translation.y.max(0.0))
                * inv_dt;

            let aabb = character.shape.compute_local_aabb();
            let extents = aabb.maxs - aabb.mins;
            #[cfg(feature = "dim2")]
            let speed = extents.x / 5.0 * inv_dt;
            #[cfg(feature = "dim3")]
            let speed = Vec2::new(extents.x, extents.z).length() / 5.0 * inv_dt;
            let y_speed = (extents.y / 30.0).max(0.1) * inv_dt;

            #[cfg(feature = "dim2")]
            for key in events.get_pressed() {
                match *key {
                    KeyCode::ArrowRight => {
                        options.velocity += Vect::X * speed;
                    }
                    KeyCode::ArrowLeft => {
                        options.velocity -= Vect::X * speed;
                    }
                    KeyCode::Space => {
                        if character.grounded {
                            options.velocity -= gravity_vel * 5.0;
                        }
                    }
                    KeyCode::ControlRight => {
                        options.velocity -= Vect::Y;
                    }
                    _ => {}
                }
            }

            #[cfg(feature = "dim3")]
            {
                let (_, rot, _): (Vec3, Quat, Vec3) = camera_transform.to_scale_rotation_translation();
                let mut rot_x = rot * Vect::X;
                let mut rot_z = rot * Vect::Z;
                rot_x.y = 0.0;
                rot_z.y = 0.0;

                for key in events.get_pressed() {
                    match *key {
                        KeyCode::ArrowRight => {
                            options.velocity += rot_x * speed;
                        }
                        KeyCode::ArrowLeft => {
                            options.velocity -= rot_x * speed;
                        }
                        KeyCode::ArrowUp => {
                            options.velocity -= rot_z * speed;
                        }
                        KeyCode::ArrowDown => {
                            options.velocity += rot_z * speed;
                        }
                        KeyCode::Space => {
                            if character.grounded {
                                options.velocity +=
                                    -gravity_vel + Vect::Y * y_speed * options.gravity_scale.sqrt();
                            }
                        }
                        KeyCode::ControlRight => {
                            options.velocity -= Vect::Y;
                        }
                        _ => {}
                    }
                }
            }

            options.velocity += gravity_vel;

            character.desired_translation = options.velocity * dt;
            character.translation = Some(options.velocity * dt);
        }
    }
}

/// Apply the character controller movements using rapier's KinematicCharacterController.
pub fn apply_character_movements(
    mut physics: ResMut<PhysicsState>,
    mut characters: Query<(
        Entity,
        &mut CharacterController,
        &mut Transform,
        Option<&ColHandle>,
    )>,
) {
    use crate::physics::rapier::control::KinematicCharacterController as RapierCharacterController;
    use crate::physics::rapier::control::CharacterLength;

    for (entity, mut character, mut transform, col_handle) in characters.iter_mut() {
        let desired = match character.translation.take() {
            Some(t) => t,
            None => continue,
        };

        let rapier_controller = RapierCharacterController {
            offset: CharacterLength::Absolute(character.offset),
            max_slope_climb_angle: character.max_slope_climb_angle,
            min_slope_slide_angle: character.min_slope_slide_angle,
            snap_to_ground: character.snap_to_ground.map(|d| {
                CharacterLength::Absolute(d)
            }),
            ..Default::default()
        };

        // Determine which collider to exclude from the sweep (the character's own collider).
        let filter = if let Some(ch) = col_handle {
            QueryFilter::default().exclude_collider(ch.0)
        } else {
            QueryFilter::default()
        };

        let dt = physics.integration_parameters.dt;
        let query_pipeline = physics.broad_phase.as_query_pipeline(
            physics.narrow_phase.query_dispatcher(),
            &physics.bodies,
            &physics.colliders,
            filter,
        );

        #[cfg(feature = "dim2")]
        let char_pose = {
            let (axis, angle) = transform.rotation.to_axis_angle();
            let angle = if axis.z < 0.0 { -angle } else { angle };
            Pose::new(transform.translation.truncate(), angle)
        };
        #[cfg(feature = "dim3")]
        let char_pose = Pose::from_parts(transform.translation, transform.rotation);

        let movement = rapier_controller.move_shape(
            dt,
            &query_pipeline,
            &*character.shape,
            &char_pose,
            desired,
            |_| {},
        );

        character.grounded = movement.grounded;
        character.effective_translation = movement.translation;

        #[cfg(feature = "dim2")]
        {
            transform.translation += movement.translation.extend(0.0);
        }
        #[cfg(feature = "dim3")]
        {
            transform.translation += movement.translation;
        }
    }
}
