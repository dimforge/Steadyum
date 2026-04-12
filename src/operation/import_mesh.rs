use crate::operation::{Operation, Operations};
use crate::physics::PhysicsState;
use bevy::prelude::*;

pub fn import_mesh(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
    operations: Res<Operations>,
) {
    let operations = &*operations;
    for op in operations.iter() {
        if let Operation::ImportMesh(_path, _shape) = op {
            // TODO: Re-implement mesh import with direct rapier collider creation.
            // let handle: Handle<Mesh> = asset_server.load(path.as_path());
            // commands
            //     .spawn(AsyncCollider(shape.clone()))
            //     .insert(handle)
            //     .insert(Transform::default())
            //     .insert(RigidBodyBundle::fixed())
            //     .insert(ColliderRenderBundle::default());
        }
    }
}

pub fn set_trimesh_flags(_physics: Res<PhysicsState>) {
    // TODO: Re-implement trimesh flag setting with direct rapier access.
    // for (_, collider) in physics.colliders.iter_mut() {
    //     if let Some(trimesh) = collider.shape_mut().as_trimesh_mut() {
    //         if let Err(topo_err) = trimesh.set_flags(TriMeshFlags::all()) {
    //             error!("topology computation error {}", topo_err);
    //         }
    //     }
    // }
}
