use bevy::prelude::*;
use super::rapier::prelude::{
    ColliderHandle as RapierColliderHandle,
    RigidBodyHandle as RapierRigidBodyHandle,
    ImpulseJointHandle as RapierImpulseJointHandle,
};

/// Component storing a rapier rigid body handle.
/// Placed on Bevy entities that have an associated rigid body in PhysicsState.
#[derive(Component, Clone, Copy, Debug)]
pub struct RbHandle(pub RapierRigidBodyHandle);

/// Component storing a rapier collider handle.
/// Placed on Bevy entities that have an associated collider in PhysicsState.
#[derive(Component, Clone, Copy, Debug)]
pub struct ColHandle(pub RapierColliderHandle);

/// Component storing a rapier impulse joint handle.
#[derive(Component, Clone, Copy, Debug)]
pub struct JointHandle(pub RapierImpulseJointHandle);
