use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use bevy_infinite_grid::*;

pub struct FloorPlugin;

impl Plugin for FloorPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "dim3")]
        {
            app.add_plugin(InfiniteGridPlugin)
                .add_startup_system(setup_floor);
        }
    }
}

fn setup_floor(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn_bundle(InfiniteGridBundle {
        grid: InfiniteGrid {
            shadow_color: None,
            fadeout_distance: 500.0,
            dot_fadeout_strength: 0.1,
            ..Default::default()
        },
        ..Default::default()
    });
}
