use bevy::prelude::*;

mod click;
mod hover;
mod track;

pub struct SelectionMousePlugin;

impl Plugin for SelectionMousePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, track::track_mouse_state);
        app.add_system_to_stage(CoreStage::Update, hover::update_hovered_entity);
        app.add_system_to_stage(CoreStage::Update, click::handle_selection_click);
    }
}
