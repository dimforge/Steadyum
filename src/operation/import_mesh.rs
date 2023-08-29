use crate::operation::{Operation, Operations};
use crate::render::{ColliderRender, ColliderRenderTargets};
use crate::utils::{ColliderRenderBundle, RigidBodyBundle};
use bevy::prelude::*;
use bevy_rapier::prelude::*;

pub fn import_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    operations: Res<Operations>,
) {
    for op in operations.iter() {
        if let Operation::ImportMesh(path, shape) = op {
            let handle: Handle<Mesh> = asset_server.load(path.as_path());
            commands
                .spawn(AsyncCollider {
                    handle: handle.clone(),
                    shape: shape.clone(),
                })
                .insert(TransformBundle::default())
                .insert(RigidBodyBundle::fixed())
                .insert(ColliderRenderBundle::default());
        }
    }
}

pub fn set_trimesh_flags(mut changed_shapes: Query<&mut Collider, Changed<Collider>>) {
    // for mut shape in changed_shapes.iter_mut() {
    //     if shape.as_trimesh().is_some() {
    //         // Check immutably first to avoid triggering bevy’s change detection.
    //         if let Some(mut trimesh) = shape.as_trimesh_mut() {
    //             if let Err(topo_err) = trimesh.set_flags(TriMeshFlags::all()) {
    //                 error!("topology computation error {}", topo_err);
    //             }
    //         }
    //     }
    // }
}
