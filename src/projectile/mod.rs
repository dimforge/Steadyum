use bevy::prelude::*;

pub(self) const ACTIVE_EPS: f32 = 1.0e-1;

mod mouse;

#[derive(Default, Clone, Resource)]
pub struct ProjectileState {
    // pub drag_local_point: Vect,
    // pub drag_plane_point: Vect,
    // pub drag_plane_normal: Vect,
    // pub dragged_entity: Option<Entity>,
    // pub mouse_body: Option<Entity>,
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProjectileState::default())
            .add_system(mouse::handle_projectile_click);
        // .add_system(mouse::handle_projectile_hover);
    }
}
