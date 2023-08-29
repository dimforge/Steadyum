use crate::operation::{Operation, Operations};
use crate::selection::SceneMouse;
use crate::ui::{ActiveMouseAction, SelectedTool, UiState};
use bevy::prelude::*;
use bevy_rapier::dynamics::{Ccd, Velocity};

use crate::projectile::ProjectileState;
use crate::utils::{ColliderBundle, RigidBodyBundle};
use bevy_rapier::geometry::ColliderMassProperties;
use bevy_rapier::prelude::Collider;

pub fn handle_projectile_click(
    mut commands: Commands,
    mut projectile_state: ResMut<ProjectileState>,
    mut mouse_action: ResMut<ActiveMouseAction>,
    mut operations: ResMut<Operations>,
    ui_state: Res<UiState>,
    scene_mouse: Res<SceneMouse>,
    mouse: Res<Input<MouseButton>>,
    transforms: Query<&Transform>,
) {
    let mut reset = false;

    match ui_state.selected_tool {
        SelectedTool::Projectile => {}
        _ => {
            reset = true;
        }
    }

    if *mouse_action != ActiveMouseAction::Projectile && *mouse_action != ActiveMouseAction::None {
        reset = true;
    }

    if !reset {
        if mouse.just_released(MouseButton::Left) {
            #[cfg(feature = "dim3")] // TODO: adapt for 2D
            if let Some((ray_pos, ray_dir)) = scene_mouse.ray {
                operations.push(Operation::AddCollider(
                    ColliderBundle {
                        mass_properties: ColliderMassProperties::Density(1000.0),
                        ..ColliderBundle::new(Collider::ball(0.3))
                    },
                    RigidBodyBundle {
                        velocity: Velocity::linear(ray_dir * 400.0),
                        ccd: Ccd::enabled(),
                        ..RigidBodyBundle::dynamic()
                    },
                    Transform::from_translation(ray_pos),
                ));
            }
        }
    }

    if reset {
        if *mouse_action == ActiveMouseAction::Projectile {
            *mouse_action = ActiveMouseAction::None;
        }
    }
}
