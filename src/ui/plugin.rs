use super::{ActiveMouseAction, UiState};
use bevy::prelude::*;

/// Plugin responsible for creating an UI for interacting, monitoring, and modifying the simulation.
pub struct RapierUiPlugin;

impl Plugin for RapierUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin)
            // .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
            .insert_resource(UiState::default())
            .insert_resource(ActiveMouseAction::None)
            .add_systems(Startup, super::load_assets)
            .add_systems(PreUpdate, super::focus_ui)
            // .add_systems(Update, super::add_missing_gizmos)
            .add_systems(Update, super::update_ui)
            .add_systems(Update, super::handle_keyboard_inputs);
    }
}
