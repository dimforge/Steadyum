use bevy::prelude::*;
use bevy_rapier::math::Vect;

mod mouse;

#[derive(Default, Clone, Resource)]
pub struct DragState {
    pub drag_local_point: Vect,
    pub drag_plane_point: Vect,
    pub drag_plane_normal: Vect,
    pub dragged_entity: Option<Entity>,
    pub mouse_body: Option<Entity>,
}

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DragState::default())
            .add_systems(Update, mouse::handle_drag_click)
            .add_systems(Update, mouse::handle_drag_hover);
    }
}
