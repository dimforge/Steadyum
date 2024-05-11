use crate::ui::UiState;
use bevy_egui::egui::Ui;
use bevy_rapier::rapier::pipeline::DebugRenderMode;
use bevy_rapier::render::DebugRenderContext;

pub(super) fn ui(ui: &mut Ui, ui_state: &mut UiState, debug_render: &mut DebugRenderContext) {
    ui.checkbox(&mut debug_render.enabled, "Enabled");
    // ui.checkbox(&mut debug_render.always_on_top, "Always on top");
    ui.separator();

    let mode = &mut debug_render.pipeline.mode;

    let items = [
        (DebugRenderMode::COLLIDER_AABBS, "AABBs"),
        (DebugRenderMode::COLLIDER_SHAPES, "Colliders"),
        (DebugRenderMode::RIGID_BODY_AXES, "Bodies"),
        (DebugRenderMode::IMPULSE_JOINTS, "Joints"),
        (DebugRenderMode::SOLVER_CONTACTS, "Solver Contacts"),
        (DebugRenderMode::CONTACTS, "Contacts"),
    ];

    for (bits, text) in items {
        let mut enabled = mode.contains(bits);
        ui.checkbox(&mut enabled, text);
        mode.set(bits, enabled);
    }

    ui.checkbox(&mut ui_state.interpolation, "Interpolation");
}
