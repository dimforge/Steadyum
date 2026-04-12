use crate::operation::{Operation, Operations};
use bevy::prelude::*;

pub fn import_scene(
    _commands: Commands,
    operations: Res<Operations>,
) {
    for op in operations.iter() {
        if let Operation::ImportScene(_scene) = op {
            todo!()
        }
    }
}
