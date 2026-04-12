use bevy::prelude::*;
use std::collections::HashMap;

use super::rapier::prelude::*;
use super::rapier::counters::Counters;

/// Main physics state resource holding all rapier sets and pipelines.
/// Replaces bevy_rapier's RapierContext.
#[derive(Resource)]
pub struct PhysicsState {
    // Core rapier sets
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,

    // Pipelines
    pub pipeline: PhysicsPipeline,
    pub broad_phase: BroadPhaseBvh,
    pub narrow_phase: NarrowPhase,
    pub ccd_solver: CCDSolver,
    pub island_manager: IslandManager,

    // Parameters
    pub integration_parameters: IntegrationParameters,
    pub gravity: super::Vector,

    // Entity <-> Handle mappings
    pub entity2body: HashMap<Entity, RigidBodyHandle>,
    pub body2entity: HashMap<RigidBodyHandle, Entity>,
    pub entity2collider: HashMap<Entity, ColliderHandle>,
    pub collider2entity: HashMap<ColliderHandle, Entity>,

    // Profiling counters
    pub counters: Counters,

    // State
    pub running: bool,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            pipeline: PhysicsPipeline::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            island_manager: IslandManager::new(),
            integration_parameters: IntegrationParameters::default(),
            #[cfg(feature = "dim2")]
            gravity: super::Vector::new(0.0, -9.81),
            #[cfg(feature = "dim3")]
            gravity: super::Vector::new(0.0, -9.81, 0.0),
            entity2body: HashMap::new(),
            body2entity: HashMap::new(),
            entity2collider: HashMap::new(),
            collider2entity: HashMap::new(),
            counters: Counters::new(true),
            running: false,
        }
    }
}

impl PhysicsState {
    /// Insert a rigid body and associate it with a Bevy entity.
    pub fn insert_body(&mut self, entity: Entity, rb: impl Into<RigidBody>) -> RigidBodyHandle {
        let handle = self.bodies.insert(rb);
        self.entity2body.insert(entity, handle);
        self.body2entity.insert(handle, entity);
        handle
    }

    /// Insert a collider attached to a rigid body and associate it with a Bevy entity.
    pub fn insert_collider_with_parent(
        &mut self,
        entity: Entity,
        collider: impl Into<Collider>,
        parent: RigidBodyHandle,
    ) -> ColliderHandle {
        let handle = self.colliders.insert_with_parent(collider, parent, &mut self.bodies);
        self.entity2collider.insert(entity, handle);
        self.collider2entity.insert(handle, entity);
        handle
    }

    /// Insert a standalone collider (no parent body) and associate it with a Bevy entity.
    pub fn insert_collider(&mut self, entity: Entity, collider: impl Into<Collider>) -> ColliderHandle {
        let handle = self.colliders.insert(collider);
        self.entity2collider.insert(entity, handle);
        self.collider2entity.insert(handle, entity);
        handle
    }

    /// Remove a rigid body and its associated colliders. Returns the entity if found.
    pub fn remove_body(&mut self, handle: RigidBodyHandle) -> Option<Entity> {
        self.bodies.remove(
            handle,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
        let entity = self.body2entity.remove(&handle);
        if let Some(e) = entity {
            self.entity2body.remove(&e);
        }
        entity
    }

    /// Remove a collider. Returns the entity if found.
    pub fn remove_collider(&mut self, handle: ColliderHandle) -> Option<Entity> {
        self.colliders.remove(handle, &mut self.island_manager, &mut self.bodies, true);
        let entity = self.collider2entity.remove(&handle);
        if let Some(e) = entity {
            self.entity2collider.remove(&e);
        }
        entity
    }

    /// Remove all bodies and colliders associated with a Bevy entity.
    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(body_handle) = self.entity2body.remove(&entity) {
            self.body2entity.remove(&body_handle);
            self.bodies.remove(
                body_handle,
                &mut self.island_manager,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            );
        }
        if let Some(col_handle) = self.entity2collider.remove(&entity) {
            self.collider2entity.remove(&col_handle);
            self.colliders.remove(col_handle, &mut self.island_manager, &mut self.bodies, true);
        }
    }

    /// Clear all physics state.
    pub fn clear(&mut self) {
        self.bodies = RigidBodySet::new();
        self.colliders = ColliderSet::new();
        self.impulse_joints = ImpulseJointSet::new();
        self.multibody_joints = MultibodyJointSet::new();
        self.broad_phase = BroadPhaseBvh::new();
        self.narrow_phase = NarrowPhase::new();
        self.ccd_solver = CCDSolver::new();
        self.island_manager = IslandManager::new();
        self.entity2body.clear();
        self.body2entity.clear();
        self.entity2collider.clear();
        self.collider2entity.clear();
    }

    /// Get the ColliderHandle for a Bevy entity.
    pub fn collider_handle(&self, entity: Entity) -> Option<ColliderHandle> {
        self.entity2collider.get(&entity).copied()
    }

    /// Get the RigidBodyHandle for a Bevy entity.
    pub fn body_handle(&self, entity: Entity) -> Option<RigidBodyHandle> {
        self.entity2body.get(&entity).copied()
    }

    /// Get the Bevy entity for a ColliderHandle.
    pub fn collider_entity(&self, handle: ColliderHandle) -> Option<Entity> {
        self.collider2entity.get(&handle).copied()
    }

    /// Get the Bevy entity for a RigidBodyHandle.
    pub fn body_entity(&self, handle: RigidBodyHandle) -> Option<Entity> {
        self.body2entity.get(&handle).copied()
    }

    /// Set a rigid body's position and immediately propagate the change to its colliders.
    /// Use this when physics is paused and you need queries/rendering to reflect the new position.
    pub fn set_body_position(&mut self, handle: RigidBodyHandle, pose: super::Pose, wake_up: bool) {
        let Some(body) = self.bodies.get_mut(handle) else {
            return;
        };
        body.set_position(pose, wake_up);
        // Collect collider handles to avoid borrow conflicts.
        let col_handles: Vec<_> = body.colliders().to_vec();
        let body_pose = *body.position();
        for col_handle in col_handles {
            if let Some(collider) = self.colliders.get_mut(col_handle) {
                let local = *collider.position_wrt_parent().unwrap_or(&super::Pose::IDENTITY);
                collider.set_position(body_pose * local);
            }
        }
    }
}
