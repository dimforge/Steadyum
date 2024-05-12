use crate::selection::{SceneMouse, SelectableSceneObject, Selection, SelectionState};
use crate::ui::{ActiveMouseAction, SelectedTool, UiState};
use bevy::prelude::*;

pub fn handle_selection_click(
    mut selection_state: ResMut<SelectionState>,
    mut mouse_action: ResMut<ActiveMouseAction>,
    scene_mouse: ResMut<SceneMouse>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    ui_state: Res<UiState>,
    mut selected_entities: Query<(Entity, &mut Selection)>,
) {
    if !selection_state.inputs_enabled {
        return;
    }

    if (*mouse_action != ActiveMouseAction::Selection && *mouse_action != ActiveMouseAction::None)
        || ui_state.selected_tool == SelectedTool::Drag
        || ui_state.selected_tool == SelectedTool::Projectile
    {
        // Clear selection.
        for (_, mut selection) in selected_entities.iter_mut() {
            selection.selected = false;
        }
        selection_state.selection_start = None;
        selection_state.selection_end = None;
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        selection_state.selection_start = scene_mouse.hovered;
    }

    if mouse.just_released(MouseButton::Left) {
        selection_state.selection_end = scene_mouse.hovered;

        if matches!(
            selection_state.selection_start,
            Some(SelectableSceneObject::SelectionShape(_))
        ) {
            return;
        }

        // Clear selection.
        if !keyboard.pressed(KeyCode::ShiftLeft) {
            for (_, mut selection) in selected_entities.iter_mut() {
                selection.selected = false;
            }
        }

        let mut selected_any = false;
        if selection_state.selection_start.is_some() {
            if let Some(hovered_object) = &scene_mouse.hovered {
                match hovered_object {
                    SelectableSceneObject::Collider(entity, _) => {
                        // Select object.
                        if let Ok((_, mut selection)) = selected_entities.get_mut(*entity) {
                            selection.selected = true;
                            selected_any = true;
                        }
                    }
                    SelectableSceneObject::SelectionShape(_) => {}
                }
            }
        }

        if selected_any {
            selection_state.selection_start = None;
            selection_state.selection_end = None;
            *mouse_action = ActiveMouseAction::None;
        }
    }
}
