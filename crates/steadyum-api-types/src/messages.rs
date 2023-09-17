use crate::objects::{ColdBodyObject, WarmBodyObject};
use crate::simulation::SimulationBounds;
use rapier::dynamics::GenericJoint;
use rapier::geometry::Aabb;
use rapier::math::{Isometry, Real};
use uuid::Uuid;

pub const PARTITIONNER_QUEUE: &str = "partitionner";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ObjectAssignment {
    pub uuid: Uuid,
    pub aabb: Aabb,
    pub warm_object: WarmBodyObject,
    pub dynamic: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug)]
pub struct ImpulseJointAssignment {
    pub body1: Uuid,
    pub body2: Uuid,
    pub joint: GenericJoint,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct BodyAssignment {
    pub uuid: Uuid,
    pub warm: WarmBodyObject,
    pub cold: ColdBodyObject, // TODO: don’t send the cold object?
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum PartitionnerMessage {
    AssignMulipleImpulseJoints {
        joints: Vec<ImpulseJointAssignment>,
    },
    ReAssignImpulseJoint(ImpulseJointAssignment),
    AssignImpulseJointTo {
        joint: ImpulseJointAssignment,
        target: SimulationBounds,
    },
    AssignMultipleObjects {
        objects: Vec<ObjectAssignment>,
    },
    AssignIsland {
        origin: String,
        objects: Vec<ObjectAssignment>,
    },
    AssignObjectTo {
        uuid: Uuid,
        origin: String,
        target: SimulationBounds,
        warm_object: WarmBodyObject,
    },
    ReAssignObject {
        uuid: Uuid,
        // TODO: replace by an Uuid?
        origin: String, // Region the object was in before.
        aabb: Aabb,
        warm_object: WarmBodyObject,
        dynamic: bool,
    },
    MoveObject {
        uuid: Uuid,
        position: Isometry<Real>,
    },
    UpdateColdObject {
        uuid: Uuid,
    },
    RemoveObject,
    StartStop {
        running: bool,
    },
    AckSteps {
        origin: String,
        stopped: bool,
    },
    AckStart {
        origin: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum RunnerMessage {
    AssignRegion {
        region: SimulationBounds,
        time_origin: u64,
    },
    AssignIsland {
        bodies: Vec<BodyAssignment>,
        impulse_joints: Vec<ImpulseJointAssignment>,
    },
    MoveObject {
        uuid: Uuid,
        position: Isometry<Real>,
    },
    UpdateColdObject {
        uuid: Uuid,
    },
    StartStop {
        running: bool,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug)]
pub struct AckSteps {
    pub step_id: u64,
}
