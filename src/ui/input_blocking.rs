use crate::selection::SelectionState;
use bevy::prelude::*;
use bevy_egui::EguiContext;

pub fn focus_ui(mut ui_context: ResMut<EguiContext>, mut selection: ResMut<SelectionState>) {
    let other_inputs_enabled = !ui_context.ctx_mut().wants_pointer_input();
    selection.inputs_enabled = other_inputs_enabled;
}
