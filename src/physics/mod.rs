pub mod components;
pub mod conversions;
pub mod plugin;
pub mod state;
pub mod systems;

// Feature-gated rapier alias
#[cfg(feature = "dim2")]
pub use rapier2d as rapier;
#[cfg(feature = "dim3")]
pub use rapier3d as rapier;

// Re-export parry through rapier
pub use rapier::parry;

// Re-export commonly used types
pub use rapier::math::{Real, Vector, AngVector, Rotation, Pose, DIM};
pub use rapier::pipeline::{
    DebugRenderBackend, DebugRenderMode, DebugRenderObject, DebugRenderPipeline, DebugRenderStyle,
};
pub use rapier::prelude::{
    BroadPhaseBvh, CCDSolver, Collider, ColliderBuilder, ColliderHandle, ColliderSet,
    DefaultBroadPhase, GenericJoint, ImpulseJointHandle, ImpulseJointSet, IntegrationParameters,
    InteractionGroups, InteractionTestMode, IslandManager, MultibodyJointSet, NarrowPhase,
    PhysicsPipeline, QueryFilter, Ray, RayIntersection, RigidBodyBuilder, RigidBodyHandle,
    RigidBodySet, RigidBodyType, SharedShape, SpringJointBuilder,
};
pub use rapier::counters::Counters;

pub use components::*;
pub use conversions::*;
pub use plugin::PhysicsPlugin;
pub use state::PhysicsState;

// Dimension-dependent type aliases
#[cfg(feature = "dim2")]
pub type Vect = bevy::math::Vec2;
#[cfg(feature = "dim3")]
pub type Vect = bevy::math::Vec3;
