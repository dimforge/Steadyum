use crate::MainCamera;
use bevy::prelude::*;
use bevy_rapier::control::{KinematicCharacterController, KinematicCharacterControllerOutput};
use bevy_rapier::geometry::Collider;
use bevy_rapier::math::Vect;
use bevy_rapier::plugin::RapierConfiguration;
use bevy_rapier::prelude::RapierContext;

pub struct ControlPlugin;

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
        app.add_systems(Update, control_characters);
    }
}

pub fn control_characters(
    events: Res<ButtonInput<KeyCode>>,
    config: Res<RapierConfiguration>,
    context: Res<RapierContext>,
    mut characters: Query<(
        &Collider,
        &mut KinematicCharacterController,
        &mut CharacterControlOptions,
        &KinematicCharacterControllerOutput,
    )>,
    cameras: Query<&GlobalTransform, With<MainCamera>>,
) {
    if !config.physics_pipeline_active {
        return;
    }

    if let Ok(camera_transform) = cameras.get_single() {
        for (collider, mut character, mut options, output) in characters.iter_mut() {
            let options = &mut *options;
            if !options.enabled {
                continue;
            }

            let dt = context.integration_parameters.dt;
            let inv_dt = context.integration_parameters.inv_dt();
            let gravity_vel = config.gravity * dt * options.gravity_scale;

            options.velocity = Vect::ZERO;
            options.velocity.y = output
                .effective_translation
                .y
                .min(output.desired_translation.y.max(0.0))
                * inv_dt;

            let collider_aabb = collider.raw.compute_local_aabb();
            #[cfg(feature = "dim2")]
            let mut speed = collider_aabb.extents().x / 5.0 * inv_dt;
            #[cfg(feature = "dim3")]
            let speed = collider_aabb.extents().xz().norm() / 5.0 * inv_dt;
            let y_speed = (collider_aabb.extents().y / 30.0).max(0.1) * inv_dt;

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
                        if output.grounded {
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
                let (_, rot, _) = camera_transform.to_scale_rotation_translation();
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
                            if output.grounded {
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

            character.translation = Some(options.velocity * dt);
        }
    }
}
