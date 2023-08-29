use crate::styling::Theme;
use crate::{UiState, Visuals};
use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_infinite_grid::InfiniteGrid;

pub fn update_dark_mode(
    mut commands: Commands,
    theme: Res<Theme>,
    mut ui_context: ResMut<EguiContext>,
    // mut floor: Query<&mut InfiniteGrid>,
) {
    ui_context.ctx_mut().set_visuals(theme.ui_visuals());
    commands.insert_resource(ClearColor(theme.background_color()));

    // for mut floor in floor.iter_mut() {
    //     floor.minor_line_color = theme.floor_minor_line_color();
    //     floor.major_line_color = theme.floor_major_line_color();
    // }
}
