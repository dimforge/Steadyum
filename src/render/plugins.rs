use crate::render::{add_collider_render_targets, CollisionShapeMeshInstances};
use crate::SteadyumStages;
use bevy::ecs::prelude::apply_deferred;
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
        // app.add_stage_before(
        //     Update,
        //     SteadyumStages::RenderStage,
        //     SystemStage::parallel(),
        // );

        app.init_resource::<CollisionShapeMeshInstances>()
            .add_systems(
                Update,
                apply_deferred
                    .after(RenderSystems::AddMissingTransforms)
                    .before(RenderSystems::CreateColliderRenders),
            )
            .add_systems(
                Update,
                add_collider_render_targets.in_set(RenderSystems::AddMissingTransforms),
            )
            .add_systems(
                Update, // SteadyumStages::RenderStage,
                super::create_collider_renders_system.in_set(RenderSystems::CreateColliderRenders),
            )
            .add_systems(
                Update, // SteadyumStages::RenderStage,
                super::add_missing_transforms.in_set(RenderSystems::AddMissingTransforms),
            );
        // .add_systems(
        //     SteadyumStages::RenderStage,
        //     super::create_collider_outline_renders_system
        //         .label(RenderSystems::CreateColliderOutlineRenders),
        // );
        // .add_systems(
        //     CoreStage::Update,
        //     super::render_joints.label(RenderSystems::RenderJoints),
        // );

        #[cfg(feature = "dim2")]
        {
            app.add_plugins(bevy_prototype_lyon::prelude::ShapePlugin);
        }

        // #[cfg(feature = "dim3")]
        // {
        //     app.add_plugins(bevy_polyline::PolylinePlugin);
        // }
    }
}
