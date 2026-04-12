use crate::render::{add_collider_render_targets, CollisionShapeMeshInstances};
use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RenderSystems {
    BeforeCommands,
    ProcessCommands,
    AddMissingTransforms,
    CreateColliderRenders,
    CreateColliderOutlineRenders,
    RenderJoints,
}

/// Plugin responsible for creating meshes to render the Rapier physics scene.
pub struct RapierRenderPlugin;

impl Plugin for RapierRenderPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                RenderSystems::BeforeCommands,
                RenderSystems::ProcessCommands,
                RenderSystems::AddMissingTransforms,
                RenderSystems::CreateColliderRenders,
                RenderSystems::CreateColliderOutlineRenders,
                RenderSystems::RenderJoints,
            )
                .chain(),
        );

        app.init_resource::<CollisionShapeMeshInstances>()
            .add_systems(
                Update,
                ApplyDeferred
                    .after(RenderSystems::AddMissingTransforms)
                    .before(RenderSystems::CreateColliderRenders),
            )
            .add_systems(
                Update,
                add_collider_render_targets.in_set(RenderSystems::AddMissingTransforms),
            )
            .add_systems(
                Update,
                super::create_collider_renders_system.in_set(RenderSystems::CreateColliderRenders),
            )
            .add_systems(
                Update,
                super::add_missing_transforms.in_set(RenderSystems::AddMissingTransforms),
            );

        app.add_systems(
            Update,
            super::create_collider_outline_renders_system
                .in_set(RenderSystems::CreateColliderOutlineRenders),
        );

        app.add_systems(
            Update,
            super::render_joints.in_set(RenderSystems::RenderJoints),
        );

        #[cfg(feature = "dim2")]
        {
            app.add_plugins(bevy_prototype_lyon::prelude::ShapePlugin);
        }
    }
}
