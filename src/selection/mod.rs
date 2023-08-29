use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_rapier::prelude::*;

pub use self::selection_shape::SelectionShape;

pub mod mouse;
mod selection_shape;
pub mod transform_gizmo;

#[derive(Copy, Clone, Component, Default)]
pub struct Selection {
    pub selected: bool,
}

impl Selection {
    pub fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SelectableSceneObject {
    #[cfg(feature = "dim2")]
    Collider(Entity, Vect),
    #[cfg(feature = "dim3")]
    Collider(Entity, RayIntersection),
    SelectionShape(Entity),
}

#[derive(Default, Copy, Clone, Debug, Resource)]
pub struct SceneMouse {
    #[cfg(feature = "dim2")]
    pub point: Option<Vect>,
    #[cfg(feature = "dim3")]
    pub ray: Option<(Vect, Vect)>,
    pub hovered: Option<SelectableSceneObject>,
}

#[derive(Default, Clone, Debug, Resource)]
pub struct SelectionState {
    pub inputs_enabled: bool,
    pub selection_start: Option<SelectableSceneObject>,
    pub selection_end: Option<SelectableSceneObject>,
}

struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectionState::default())
            .insert_resource(SceneMouse::default())
            .add_system(add_missing_selection_components);
    }
}

fn add_missing_selection_components(
    mut commands: Commands,
    missing: Query<Entity, (Without<Selection>, Or<(With<Collider>, With<RigidBody>)>)>,
) {
    for entity in missing.iter() {
        commands.entity(entity).insert(Selection::default());
    }
}

pub struct SelectionPlugins;

impl PluginGroup for SelectionPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SelectionPlugin)
            .add(mouse::SelectionMousePlugin)
            .add(transform_gizmo::TransformGizmoPlugin::default())
    }
}
