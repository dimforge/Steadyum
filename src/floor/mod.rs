use bevy::prelude::*;
use bevy_infinite_grid::*;

pub struct FloorPlugin;

impl Plugin for FloorPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "dim3")]
        {
            app.add_plugins(InfiniteGridPlugin)
                .add_systems(Startup, setup_floor);
        }
    }
}

fn setup_floor(mut commands: Commands, _meshes: ResMut<Assets<Mesh>>) {
    commands
        .spawn(InfiniteGridBundle {
            settings: InfiniteGridSettings {
                // shadow_color: None,
                fadeout_distance: 500.0,
                dot_fadeout_strength: 0.1,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Infinite Grid"));
}
