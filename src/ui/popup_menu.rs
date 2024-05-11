use bevy::window::Window;
use bevy_egui::EguiContexts;
use bevy_rapier::plugin::{RapierConfiguration, RapierContext};

pub(super) fn ui(
    _window: &Window,
    _ui_context: &mut EguiContexts,
    _physics_context: &mut RapierContext,
    _physics_config: &mut RapierConfiguration,
) {
}
