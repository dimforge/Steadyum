use crate::messages::BodyAssignment;
use crate::simulation::SimulationBounds;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PARTITIONNER_PORT: usize = 3535;
pub const RUNNER_INITIALIZED_ENDPOINT: &str = "/initialized";
pub const ASSIGN_RUNNER_ENDPOINT: &str = "/region";
pub const INSERT_OBJECTS_ENDPOINT: &str = "/insert";
pub const LIST_REGIONS_ENDPOINT: &str = "/list_regions";
pub const START_STOP_ENDPOINT: &str = "/start_stop";

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunnerInitializedRequest {
    pub uuid: Uuid,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssignRunnerRequest {
    pub region: SimulationBounds,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssignRunnerResponse {
    pub region: SimulationBounds,
    pub uuid: Uuid,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InsertObjectsRequest {
    pub bodies: Vec<BodyAssignment>,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct StartStopRequest {
    pub running: bool,
}
