use crate::selection::SelectionState;
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;

pub fn focus_ui(egui_wants_input: Res<EguiWantsInput>, mut selection: ResMut<SelectionState>) {
    selection.inputs_enabled = !egui_wants_input.wants_any_pointer_input();
}
