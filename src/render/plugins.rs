use crate::render::{add_collider_render_targets, instancing};
use crate::SteadyumStages;
use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum RenderSystems {
    AddMissingTransforms,
    CreateColliderRenders,
    CreateColliderOutlineRenders,
    RenderJoints,
}

/// Plugin responsible for creating meshes to render the Rapier physics scene.
pub struct RapierRenderPlugin;

impl Plugin for RapierRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            CoreStage::PostUpdate,
            SteadyumStages::RenderStage,
            SystemStage::parallel(),
        );

        app // .add_plugin(bevy_prototype_debug_lines::DebugLinesPlugin::with_depth_test(false))
            .add_plugin(instancing::InstancingMaterialPlugin)
            .add_system_to_stage(CoreStage::Update, add_collider_render_targets)
            .add_system_to_stage(
                SteadyumStages::RenderStage,
                super::create_collider_renders_system.label(RenderSystems::CreateColliderRenders),
            )
            .add_system_to_stage(
                SteadyumStages::RenderStage,
                super::add_missing_transforms.label(RenderSystems::AddMissingTransforms),
            )
            .add_system_to_stage(
                SteadyumStages::RenderStage,
                super::create_collider_outline_renders_system
                    .label(RenderSystems::CreateColliderOutlineRenders),
            );
        // .add_system_to_stage(
        //     CoreStage::PreUpdate,
        //     super::render_joints.label(RenderSystems::RenderJoints),
        // );

        #[cfg(feature = "dim2")]
        {
            app.add_plugin(bevy_prototype_lyon::prelude::ShapePlugin);
        }

        #[cfg(feature = "dim3")]
        {
            app.add_plugin(bevy_polyline::PolylinePlugin);
        }
    }
}
