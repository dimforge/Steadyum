use crate::operation::{Operation, Operations};
use crate::styling::ColorGenerator;
use crate::utils::{ColliderBundle, ColliderRenderBundle, RigidBodyBundle};
use bevy::prelude::*;
use bevy_rapier::prelude::*;
use std::collections::HashMap;

pub fn import_scene(
    mut commands: Commands,
    operations: Res<Operations>,
    mut colors: ResMut<ColorGenerator>,
) {
    for op in operations.iter() {
        if let Operation::ImportScene(scene) = op {
            todo!()
        }
    }
}
