use bevy::prelude::*;

use crate::physics::PhysicsState;
use crate::utils::{ColliderBundle, RigidBodyBundle};
use std::path::PathBuf;

/// Describes the shape computation strategy for imported meshes.
#[cfg(feature = "dim3")]
#[derive(Clone, Debug)]
pub enum ComputedColliderShape {
    /// Use the mesh as a triangle mesh collider.
    TriMesh,
    /// Compute a convex hull from the mesh vertices.
    ConvexHull,
    /// Compute a convex decomposition from the mesh.
    ConvexDecomposition,
}

pub enum Operation {
    #[cfg(feature = "dim3")]
    ImportMesh(PathBuf, ComputedColliderShape),
    AddPlane, // { start: Point<f32>, stop: Point<f32> },
    AddCollider(ColliderBundle, RigidBodyBundle, Transform),
    AddIntersection,
    ExportScene(PathBuf),
    ImportScene(PhysicsState),
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
