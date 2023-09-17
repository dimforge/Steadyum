use crate::messages::BodyAssignment;
use crate::objects::RegionList;
use crate::partitionner::{
    AssignRunnerRequest, AssignRunnerResponse, InsertObjectsRequest, RunnerInitializedRequest,
    StartStopRequest, ASSIGN_RUNNER_ENDPOINT, INSERT_OBJECTS_ENDPOINT, LIST_REGIONS_ENDPOINT,
    PARTITIONNER_PORT, RUNNER_INITIALIZED_ENDPOINT, START_STOP_ENDPOINT,
};
use crate::simulation::SimulationBounds;
use reqwest::blocking::Client;
use uuid::Uuid;

pub struct PartitionnerServer {
    client: Client,
}

impl PartitionnerServer {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::new();
        Ok(Self { client })
    }

    pub fn put_runner_initialized(&self, uuid: Uuid) -> anyhow::Result<()> {
        let body = RunnerInitializedRequest { uuid };
        self.client
            .post(endpoint(RUNNER_INITIALIZED_ENDPOINT))
            .json(&body)
            .send()?;
        Ok(())
    }

    pub fn allocate_runner(&self, region: SimulationBounds) -> anyhow::Result<Uuid> {
        let body = AssignRunnerRequest { region };
        let raw_response = self
            .client
            .post(endpoint(ASSIGN_RUNNER_ENDPOINT))
            .json(&body)
            .send()?;
        let response: AssignRunnerResponse = raw_response.json()?;
        Ok(response.uuid)
    }

    pub fn insert_objects(&self, bodies: Vec<BodyAssignment>) -> anyhow::Result<()> {
        let body = InsertObjectsRequest { bodies };
        self.client
            .post(endpoint(INSERT_OBJECTS_ENDPOINT))
            .json(&body)
            .send()?;
        Ok(())
    }

    pub fn list_regions(&self) -> anyhow::Result<RegionList> {
        let raw_response = self.client.get(endpoint(LIST_REGIONS_ENDPOINT)).send()?;
        Ok(raw_response.json()?)
    }

    pub fn set_running(&self, running: bool) -> anyhow::Result<()> {
        let body = StartStopRequest { running };
        self.client
            .post(endpoint(START_STOP_ENDPOINT))
            .json(&body)
            .send()?;
        Ok(())
    }
}

fn endpoint(endpoint: &str) -> String {
    format!("http://localhost:{PARTITIONNER_PORT}{endpoint}")
}
