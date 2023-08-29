use steadyum_api_types::messages::PARTITIONNER_QUEUE;
use steadyum_api_types::zenoh::ZenohContext;
use zenoh::prelude::sync::SyncResolve;
use zenoh::publication::Publisher;

pub struct Neighbors<'a> {
    pub partitionner: Publisher<'a>,
}

impl<'a> Neighbors<'a> {
    pub fn new(zenoh: &'a ZenohContext) -> Self {
        let partitionner = zenoh
            .session
            .declare_publisher(PARTITIONNER_QUEUE)
            .res_sync()
            .expect("Runner error 2");

        Self { partitionner }
    }
}
