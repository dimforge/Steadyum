use super::{ActiveMouseAction, UiState};
use bevy::prelude::*;

/// Plugin responsible for creating an UI for interacting, monitoring, and modifying the simulation.
pub struct RapierUiPlugin;

impl Plugin for RapierUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_egui::EguiPlugin)
            // .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
            .insert_resource(UiState::default())
            .insert_resource(ActiveMouseAction::None)
            .add_startup_system(super::load_assets)
            .add_system_to_stage(CoreStage::PreUpdate, super::focus_ui)
            .add_system(super::add_missing_gizmos)
            .add_system(super::update_ui)
            .add_system(super::handle_keyboard_inputs);
    }
}
