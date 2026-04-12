use super::{ActiveMouseAction, UiState};
use super::debug_render::DebugRenderState;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

/// Plugin responsible for creating an UI for interacting, monitoring, and modifying the simulation.
pub struct RapierUiPlugin;

impl Plugin for RapierUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin::default())
            .insert_resource(UiState::default())
            .insert_resource(ActiveMouseAction::None)
            .insert_resource(DebugRenderState::default())
            .add_systems(PreUpdate, super::focus_ui)
            .add_systems(Update, super::add_missing_gizmos)
            .add_systems(EguiPrimaryContextPass, super::update_ui)
            .add_systems(Update, super::handle_keyboard_inputs)
            .add_systems(Update, super::debug_render::render_debug);
    }
}
