use crate::operation::{self, Operations};
use bevy::prelude::*;

pub struct RapierOperationsPlugin;

impl Plugin for RapierOperationsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Operations::default())
            .add_system_to_stage(CoreStage::Last, clear_operations)
            .add_system_to_stage(CoreStage::PostUpdate, operation::add_plane)
            .add_system_to_stage(CoreStage::PostUpdate, operation::add_collision_shape)
            .add_system_to_stage(CoreStage::PostUpdate, operation::add_intersection)
            .add_system_to_stage(CoreStage::PostUpdate, operation::update_intersection)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                operation::import_scene.after(operation::clear_scene),
            )
            .add_system_to_stage(CoreStage::PostUpdate, operation::clear_scene);
        #[cfg(feature = "dim3")]
        {
            app.add_system_to_stage(CoreStage::PostUpdate, operation::set_trimesh_flags)
                .add_system_to_stage(CoreStage::PostUpdate, operation::import_mesh);
        }
    }
}

fn clear_operations(mut operations: ResMut<Operations>) {
    operations.clear();
}
