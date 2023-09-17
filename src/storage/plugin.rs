use crate::builtin_scenes::BuiltinScene;
use bevy::prelude::*;
use bevy_rapier::prelude::Velocity;
use bevy_rapier::rapier::prelude::{GenericJoint, Real, RigidBodyHandle};
use steadyum_api_types::objects::{ColdBodyObject, WarmBodyObject};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Component)]
pub struct External<T>(pub T);

// FIXME: remove this
#[derive(Copy, Clone, Debug, Component)]
pub struct IsAwarenessBody {
    pub last_update: Real,
    pub velocity: Velocity,
    pub update_id: u32,
}

pub struct StoragePlugin;

#[cfg(target_arch = "wasm32")]
impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {}
}

#[cfg(not(target_arch = "wasm32"))]
impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        use super::systems;

        let context = super::db::spawn_db_thread();
        app.insert_resource(context)
            .add_system_to_stage(CoreStage::Last, systems::publish_new_objects_to_kvs)
            .add_system_to_stage(
                CoreStage::Last,
                systems::write_selected_object_position_to_kvs,
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                systems::read_object_positions_from_kvs,
            )
            .add_system_to_stage(CoreStage::PreUpdate, systems::update_start_stop)
            .add_system_to_stage(CoreStage::PreUpdate, systems::add_interpolation_components)
            .add_system(systems::update_camera_pos)
            .add_system(systems::step_interpolations)
            .add_system(systems::update_physics_progress)
            .add_system(systems::integrate_kinematic_animations)
            .add_system(systems::read_new_objects_from_kvs)
            .add_system_to_stage(CoreStage::Last, systems::write_modified_cold_objects_to_kvs);
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HandleOrUuid {
    Handle(RigidBodyHandle),
    Uuid(Uuid),
}

impl From<Uuid> for HandleOrUuid {
    fn from(value: Uuid) -> Self {
        Self::Uuid(value)
    }
}

impl From<RigidBodyHandle> for HandleOrUuid {
    fn from(value: RigidBodyHandle) -> Self {
        Self::Handle(value)
    }
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SaveFileData {
    pub objects: Vec<(HandleOrUuid, ColdBodyObject, WarmBodyObject)>,
    pub impulse_joints: Vec<(HandleOrUuid, HandleOrUuid, GenericJoint)>,
}

impl From<BuiltinScene> for SaveFileData {
    fn from(scene: BuiltinScene) -> Self {
        let mut result = SaveFileData::default();

        for (_, collider) in scene.context.colliders.iter() {
            let parent = collider
                .parent()
                .expect("Parentless colliders are not supported yet.");
            let body = &scene.context.bodies[parent];
            let warm_object = WarmBodyObject::from_body(body, 0);
            let mut cold_object = ColdBodyObject::from_body_collider(body, collider);
            if let Some(animations) = scene.animations.get(&parent) {
                cold_object.animations = animations.clone();
            }
            result
                .objects
                .push((parent.into(), cold_object, warm_object));
        }

        for (_, joint) in scene.context.impulse_joints.iter() {
            result
                .impulse_joints
                .push((joint.body1.into(), joint.body2.into(), joint.data));
        }

        result
    }
}

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct ExistsInDb {
    pub uuid: Uuid,
}
