use amiquip::Publish;
use std::collections::HashMap;
use steadyum_api_types::messages::{AckSteps, RunnerMessage, PARTITIONNER_QUEUE};
use steadyum_api_types::region_db::PartitionnerServer;
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::{runner_zenoh_ack_key, runner_zenoh_commands_key, ZenohContext};
use uuid::Uuid;
use zenoh::prelude::sync::SyncResolve;
use zenoh::prelude::SplitBuffer;
use zenoh::publication::Publisher;
use zenoh::sample::Sample;
use zenoh::subscriber::Subscriber;

pub struct NeighborAckState<'a> {
    queue: Subscriber<'a, flume::Receiver<Sample>>,
    step_id: Option<u64>,
}

pub struct NeighborRunner<'a> {
    pub queue: Publisher<'a>,
    pub uuid: Uuid,
}

impl<'a> NeighborRunner<'a> {
    pub fn send(&self, message: &RunnerMessage) -> anyhow::Result<()> {
        let data = serde_json::to_string(message)?;
        Ok(self.queue.put(data).res_sync().unwrap())
    }
}

pub struct Neighbors<'a> {
    pub zenoh: &'a ZenohContext,
    pub runners: HashMap<SimulationBounds, NeighborRunner<'a>>,
    pub neighbor_acks: Vec<NeighborAckState<'a>>,
}

impl<'a> Neighbors<'a> {
    pub fn new(zenoh: &'a ZenohContext) -> Self {
        Self {
            zenoh,
            runners: HashMap::default(),
            neighbor_acks: vec![],
        }
    }

    pub fn init_neighbor_acks(&mut self, region: SimulationBounds) -> anyhow::Result<()> {
        for nbh in region.all_neighbors() {
            let key = runner_zenoh_ack_key(&nbh);
            let queue = self
                .zenoh
                .session
                .declare_subscriber(&key)
                .res_sync()
                .unwrap();
            self.neighbor_acks.push(NeighborAckState {
                queue,
                step_id: None,
            });
        }

        Ok(())
    }

    pub fn update_neighbor_acks(&mut self) -> anyhow::Result<Option<u64>> {
        for ack_state in &mut self.neighbor_acks {
            while let Ok(sample) = ack_state.queue.try_recv() {
                let payload = sample.value.payload.contiguous();
                let body = String::from_utf8_lossy(&payload);
                let ack: AckSteps = serde_json::from_str(&body).unwrap();
                ack_state.step_id = Some(ack.step_id)
            }
        }

        Ok(self
            .neighbor_acks
            .iter()
            .filter_map(|ack| ack.step_id)
            .min())
    }

    pub fn fetch_or_spawn_neighbor(
        &mut self,
        db: &PartitionnerServer,
        region: SimulationBounds,
    ) -> &NeighborRunner<'a> {
        let result = self.runners.entry(region).or_insert_with(|| {
            let uuid = db.allocate_runner(region).unwrap();
            let zenoh_key = runner_zenoh_commands_key(uuid);
            let queue = self
                .zenoh
                .session
                .declare_publisher(zenoh_key)
                .res_sync()
                .unwrap();
            NeighborRunner { queue, uuid }
        });
        &*result
    }
}
