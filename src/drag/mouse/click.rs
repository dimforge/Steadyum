use crate::selection::SceneMouse;
use crate::ui::{ActiveMouseAction, SelectedTool, UiState};
use bevy::prelude::*;

use crate::drag::DragState;
use crate::physics::{
    PhysicsState, RbHandle, RigidBodyBuilder, SpringJointBuilder,
};
#[cfg(feature = "dim3")]
use crate::selection::SelectableSceneObject;

pub fn handle_drag_click(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    mut mouse_action: ResMut<ActiveMouseAction>,
    mut physics: ResMut<PhysicsState>,
    ui_state: Res<UiState>,
    scene_mouse: Res<SceneMouse>,
    mouse: Res<ButtonInput<MouseButton>>,
    transforms: Query<&Transform>,
) {
    let mut reset = false;

    match ui_state.selected_tool {
        SelectedTool::Drag => {}
        _ => {
            reset = true;
        }
    }

    if *mouse_action != ActiveMouseAction::Insertion && *mouse_action != ActiveMouseAction::None {
        reset = true;
    }

    if !reset {
        if mouse.just_pressed(MouseButton::Left) {
            #[cfg(feature = "dim2")]
            {}

            #[cfg(feature = "dim3")]
            {
                if let Some(SelectableSceneObject::Collider(entity, inter)) = scene_mouse.hovered {
                    let (ray_pos, ray_dir) = scene_mouse.ray.unwrap();
                    let hit_point = ray_pos + ray_dir * inter.time_of_impact;
                    let transform = transforms.get(entity).unwrap();
                    drag_state.drag_plane_normal = -ray_dir;
                    drag_state.drag_plane_point = hit_point;
                    // TODO: should be in the local-space of the parent.
                    drag_state.drag_local_point =
                        transform.rotation.inverse() * (hit_point - transform.translation);
                    drag_state.dragged_entity = Some(entity);

                    // Clean up any previous drag body/joint.
                    cleanup_drag(&mut commands, &mut drag_state, &mut physics);

                    // Find the body handle of the dragged entity.
                    let dragged_body_handle = physics.body_handle(entity);

                    if let Some(dragged_body_handle) = dragged_body_handle {
                        // Spawn a temporary kinematic rigid body at the click point.
                        let mouse_entity = commands.spawn(Transform::from_translation(hit_point)).id();

                        let mouse_body_handle = physics.insert_body(
                            mouse_entity,
                            RigidBodyBuilder::kinematic_position_based()
                                .translation(hit_point),
                        );
                        commands.entity(mouse_entity).insert(RbHandle(mouse_body_handle));

                        // Create a spring joint between the dragged body and the mouse body.
                        let spring_joint = SpringJointBuilder::new(0.0, 100.0, 100.0)
                            .local_anchor1(drag_state.drag_local_point)
                            .build();

                        let joint_handle = physics.impulse_joints.insert(
                            dragged_body_handle,
                            mouse_body_handle,
                            spring_joint,
                            true,
                        );

                        drag_state.mouse_body = Some(mouse_entity);
                        drag_state.spring_joint = Some(joint_handle);
                    }
                }
            }
        }

        if mouse.just_released(MouseButton::Left) {
            reset = true;
        }
    }

    if reset {
        if *mouse_action == ActiveMouseAction::Drag {
            *mouse_action = ActiveMouseAction::None;
        }

        cleanup_drag(&mut commands, &mut drag_state, &mut physics);
    }
}

fn cleanup_drag(
    commands: &mut Commands,
    drag_state: &mut DragState,
    physics: &mut PhysicsState,
) {
    // Remove the spring joint from physics.
    if let Some(joint_handle) = drag_state.spring_joint.take() {
        physics.impulse_joints.remove(joint_handle, true);
    }

    // Remove the temporary kinematic body from physics and despawn the entity.
    if let Some(entity) = drag_state.mouse_body.take() {
        physics.remove_entity(entity);
        commands.entity(entity).despawn();
    }

    drag_state.dragged_entity = None;
}
