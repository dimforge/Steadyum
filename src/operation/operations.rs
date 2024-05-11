use bevy::prelude::*;

use crate::utils::{ColliderBundle, RigidBodyBundle};
#[cfg(feature = "dim3")]
use bevy_rapier::geometry::ComputedColliderShape;
use bevy_rapier::plugin::RapierContext;
use std::path::PathBuf;

pub enum Operation {
    #[cfg(feature = "dim3")]
    ImportMesh(PathBuf, ComputedColliderShape),
    AddPlane, // { start: Point<f32>, stop: Point<f32> },
    AddCollider(ColliderBundle, RigidBodyBundle, Transform),
    AddIntersection,
    ExportScene(PathBuf),
    ImportScene(RapierContext),
    ClearScene,
}

#[derive(Resource)]
pub struct Operations {
    stack: Vec<Operation>,
}

impl Default for Operations {
    fn default() -> Self {
        Self::new()
    }
}

impl Operations {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn push(&mut self, command: Operation) {
        self.stack.push(command);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Operation> {
        self.stack.iter()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }
}
