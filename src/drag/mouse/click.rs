use crate::selection::SceneMouse;
use crate::ui::{ActiveMouseAction, SelectedTool, UiState};
use bevy::prelude::*;

use crate::drag::DragState;
#[cfg(feature = "dim3")]
use {
    crate::selection::SelectableSceneObject,
    bevy_rapier::dynamics::{ImpulseJoint, RigidBody, SpringJointBuilder},
};

pub fn handle_drag_click(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    mut mouse_action: ResMut<ActiveMouseAction>,
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
                    let transform = transforms.get(entity).unwrap();
                    drag_state.drag_plane_normal = -scene_mouse.ray.unwrap().1;
                    drag_state.drag_plane_point = inter.point;
                    // TODO: should be in the local-space of the parent.
                    drag_state.drag_local_point =
                        transform.rotation.inverse() * (inter.point - transform.translation);
                    drag_state.dragged_entity = Some(entity);

                    if let Some(entity) = drag_state.mouse_body {
                        // Despawn the previous body if there was one, this will
                        // also delete the attached joint.
                        commands.entity(entity).despawn();
                    }

                    // Spawn a dummy rigid-body, and attach the joint.
                    let entity = commands
                        .spawn(RigidBody::KinematicPositionBased)
                        .insert(Transform::from_translation(inter.point))
                        .insert(GlobalTransform::default())
                        .insert(ImpulseJoint::new(
                            entity,
                            // TODO: adjust based on the rigid-body.
                            SpringJointBuilder::new(0.0, 100.0, 100.0)
                                .local_anchor1(drag_state.drag_local_point),
                        ))
                        .id();
                    drag_state.mouse_body = Some(entity);
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

        if let Some(entity) = drag_state.mouse_body {
            commands.entity(entity).despawn();
        }

        drag_state.dragged_entity = None;
        drag_state.mouse_body = None;
    }
}
