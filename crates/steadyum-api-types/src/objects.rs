use crate::kinematic::{KinematicAnimations, KinematicCurve};
use rapier::geometry::Aabb;
use rapier::math::{AngVector, Isometry, Point, Real, Vector};
use rapier::parry::bounding_volume::BoundingSphere;
use rapier::prelude::{Collider, ColliderShape, RigidBody, RigidBodyType};
use std::time::Instant;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BodyPositionObject {
    pub uuid: Uuid,
    pub timestamp: u64,
    pub position: Isometry<Real>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WarmBodyObjectSet {
    pub timestamp: u64,
    pub objects: Vec<BodyPositionObject>,
}

#[derive(Copy, Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct WarmBodyObject {
    pub timestamp: u64,
    pub position: Isometry<Real>,
    pub linvel: Vector<Real>,
    pub angvel: AngVector<Real>,
}

impl WarmBodyObject {
    pub fn from_body(body: &RigidBody, timestamp: u64) -> Self {
        Self {
            timestamp,
            position: *body.position(),
            linvel: *body.linvel(),
            angvel: body.angvel().clone(),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ColdBodyObject {
    pub body_type: RigidBodyType,
    pub shape: ColliderShape,
    pub animations: KinematicAnimations,
}

impl ColdBodyObject {
    pub fn from_body_collider(body: &RigidBody, collider: &Collider) -> Self {
        Self {
            body_type: body.body_type(),
            shape: collider.shared_shape().clone(),
            animations: KinematicAnimations::default(),
        }
    }
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WatchedObjects {
    pub objects: Vec<(Uuid, BoundingSphere)>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct RegionList {
    pub keys: Vec<String>,
    pub ports: Vec<u32>,
}
