use bevy::prelude::*;

mod click;
mod hover;
mod track;

pub struct SelectionMousePlugin;

impl Plugin for SelectionMousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, track::track_mouse_state);
        app.add_systems(Update, hover::update_hovered_entity);
        app.add_systems(Update, click::handle_selection_click);
    }
}
