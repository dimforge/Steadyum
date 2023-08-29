use crate::simulation::{SimulationBounds, SimulationBoundsU8};
use mongodb::bson::doc;
use mongodb::sync::{Client, Collection, Database};
use rapier::math::DIM;
use uuid::Uuid;

#[derive(Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunnerDocument {
    pub uuid: Uuid,
    pub region: Option<SimulationBoundsU8>,
    pub allocated: bool,
}

#[derive(Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct AllocateRunnerDocument {
    pub allocated: bool,
    pub region: SimulationBoundsU8,
}

pub struct DbContext {
    client: Client,
    database: Database,
    runners: Collection<RunnerDocument>,
}

impl DbContext {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::with_uri_str("mongodb://localhost:27017")?;
        let database = client.database("steadyum");
        let runners = database.collection::<RunnerDocument>("runners");

        Ok(Self {
            client,
            database,
            runners,
        })
    }

    pub fn put_new_runner(&self, uuid: Uuid) -> anyhow::Result<()> {
        let doc = RunnerDocument {
            uuid,
            region: None,
            allocated: false,
        };
        self.runners.insert_one(doc, None)?;
        Ok(())
    }

    pub fn allocate_runner(&self, region: SimulationBounds) -> anyhow::Result<Uuid> {
        let region_key = region.as_bytes();
        let allocated_value = AllocateRunnerDocument {
            allocated: true,
            region: region_key,
        };

        loop {
            let result = self.runners.find_one_and_update(
                doc! { "allocated": false },
                doc! {
                    "$set": allocated_value,
                },
                None,
            );

            match result {
                Ok(Some(result)) => return Ok(result.uuid),
                Ok(None) => {
                    /* Wait for a runner to be available. */
                    dbg!("Waiting for runner to become available.");
                }
                Err(_) => {
                    // Duplicate region key, it already exists.
                    let region = self.runners.find_one(doc! { "region": region_key }, None)?;
                    return Ok(region.unwrap().uuid);
                }
            }
        }
    }
}
