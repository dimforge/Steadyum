use crate::insertion::{InsertionPreview, InsertionState, InsertionStep};
use crate::operation::Operations;
use crate::selection::SceneMouse;
use crate::ui::{ActiveMouseAction, SelectedTool, UiState};
use bevy::prelude::*;

#[cfg(feature = "dim3")]
use {crate::selection::SelectableSceneObject, bevy_rapier::rapier::utils::WBasis};

pub fn handle_insertion_click(
    mut commands: Commands,
    mut insertion_state: ResMut<InsertionState>,
    mut operations: ResMut<Operations>,
    mut mouse_action: ResMut<ActiveMouseAction>,
    ui_state: Res<UiState>,
    scene_mouse: Res<SceneMouse>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    preview: Query<(Entity, &InsertionPreview)>,
    transforms: Query<&Transform>,
) {
    let mut reset = false;

    match ui_state.selected_tool {
        SelectedTool::AddBall
        | SelectedTool::AddCuboid
        | SelectedTool::AddCapsule
        | SelectedTool::AddHeightfield => {}
        #[cfg(feature = "dim3")]
        SelectedTool::AddCone | SelectedTool::AddCylinder => {}
        _ => {
            reset = true;
        }
    }

    if *mouse_action != ActiveMouseAction::Insertion && *mouse_action != ActiveMouseAction::None {
        reset = true;
    }

    if keyboard.pressed(KeyCode::ShiftLeft) {
        insertion_state.unlocked_scaling = true;
    } else {
        insertion_state.unlocked_scaling = false;
    }

    let (preview_entity, _) = preview.get_single().unwrap();

    if !reset {
        if mouse.just_pressed(MouseButton::Left) {
            #[cfg(feature = "dim2")]
            match insertion_state.step {
                None => {
                    if scene_mouse.hovered.is_none() {
                        if let Some(point) = scene_mouse.point {
                            // Cast against the ground.
                            insertion_state.start_point = point;
                            insertion_state.basis = [Vec2::X, Vec2::Y];
                            insertion_state.step = Some(InsertionStep::Basis);
                            insertion_state.on_empty_ground = false;
                        }
                    }

                    if insertion_state.step.is_some() {
                        insertion_state.end_point = insertion_state.start_point;

                        insertion_state.set_tool(ui_state.selected_tool);
                        commands
                            .entity(preview_entity)
                            .insert(Visibility::Visible)
                            .insert(insertion_state.transform())
                            .insert(GlobalTransform::default());
                    } else {
                        // Reset to a random tool we don’t use, just to reset the selection shape.
                        insertion_state.set_tool(SelectedTool::Translate);
                    }
                }
                _ => {}
            }

            #[cfg(feature = "dim3")]
            match insertion_state.step {
                None => {
                    if let Some(SelectableSceneObject::Collider(entity, inter)) =
                        scene_mouse.hovered
                    {
                        let transform = transforms.get(entity).unwrap();
                        let local_normal: na::Vector3<_> =
                            (transform.rotation.inverse() * inter.normal).into();
                        let local_basis_xz = local_normal.orthonormal_basis();
                        let local_basis: [Vec3; 3] = [
                            local_basis_xz[1].into(),
                            local_normal.into(),
                            local_basis_xz[0].into(),
                        ];
                        let basis = [
                            transform.rotation * local_basis[0],
                            transform.rotation * local_basis[1],
                            transform.rotation * local_basis[2],
                        ];

                        insertion_state.start_point = inter.point;
                        insertion_state.basis = basis;
                        insertion_state.step = Some(InsertionStep::Basis);
                        insertion_state.on_empty_ground = false;
                    } else if scene_mouse.hovered.is_none() {
                        if let Some((ray_pos, ray_dir)) = scene_mouse.ray {
                            // Cast against the ground.
                            if ray_dir.y.abs() > 1.0e-3 {
                                let ground_hit = ray_pos.y / -ray_dir.y;
                                insertion_state.start_point = ray_pos + ray_dir * ground_hit;
                                insertion_state.basis = [Vec3::X, Vec3::Y, Vec3::Z];
                                insertion_state.step = Some(InsertionStep::Basis);
                                insertion_state.on_empty_ground = true;
                            }
                        }
                    }

                    if insertion_state.step.is_some() {
                        insertion_state.end_point = insertion_state.start_point;
                        insertion_state.height = 0.0;

                        insertion_state.set_tool(ui_state.selected_tool);
                        commands
                            .entity(preview_entity)
                            .insert(Visibility::Visible)
                            .insert(insertion_state.transform())
                            .insert(GlobalTransform::default());
                    } else {
                        // Reset to a random tool we don’t use, just to reset the selection shape.
                        insertion_state.set_tool(SelectedTool::Translate);
                    }
                }
                Some(InsertionStep::Height) => {
                    if !insertion_state.intersects_environment {
                        insertion_state.step = Some(InsertionStep::Orientation);
                    }
                }
                _ => {}
            }
        }

        if mouse.just_pressed(MouseButton::Right) {
            reset = true;
        }

        if mouse.just_released(MouseButton::Left) {
            dbg!("Here");
            match insertion_state.step {
                Some(InsertionStep::Basis) => {
                    if insertion_state.intersects_environment
                        || (insertion_state.end_point - insertion_state.start_point).length()
                            < crate::insertion::ACTIVE_EPS
                    {
                        reset = true;
                    } else {
                        #[cfg(feature = "dim2")]
                        {
                            operations.push(insertion_state.operation());
                            reset = true;
                        }
                        #[cfg(feature = "dim3")]
                        {
                            insertion_state.step = Some(InsertionStep::Height);
                        }
                    }
                }
                #[cfg(feature = "dim3")]
                Some(InsertionStep::Height) => { /* Noting to do, but don’t reset. */ }
                #[cfg(feature = "dim3")]
                Some(InsertionStep::Orientation) => {
                    if !insertion_state.intersects_environment {
                        operations.push(insertion_state.operation());
                        reset = true;
                    }
                }
                _ => {
                    reset = true;
                }
            }
        }
    }

    if reset {
        if *mouse_action == ActiveMouseAction::Insertion {
            *mouse_action = ActiveMouseAction::None;
        }
        insertion_state.step = None;
        commands.entity(preview_entity).insert(Visibility::Hidden);
    }
}
