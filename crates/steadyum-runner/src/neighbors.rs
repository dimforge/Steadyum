use amiquip::Publish;
use std::collections::HashMap;
use steadyum_api_types::messages::PARTITIONNER_QUEUE;
use steadyum_api_types::simulation::SimulationBounds;
use steadyum_api_types::zenoh::ZenohContext;
use uuid::Uuid;
use zenoh::prelude::sync::SyncResolve;
use zenoh::publication::Publisher;

pub struct NeighborRunner<'a> {
    pub queue: Publisher<'a>,
    pub uuid: Uuid,
}

pub struct Neighbors<'a> {
    pub partitionner: Publisher<'a>,
    pub runners: HashMap<SimulationBounds, NeighborRunner<'a>>,
}

impl<'a> Neighbors<'a> {
    pub fn new(zenoh: &'a ZenohContext) -> Self {
        let partitionner = zenoh
            .session
            .declare_publisher(PARTITIONNER_QUEUE)
            .res_sync()
            .expect("Runner error 2");

        Self {
            partitionner,
            runners: HashMap::default(),
        }
    }
}
