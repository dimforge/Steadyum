use serde::Serialize;
use zenoh::prelude::sync::Config;
use zenoh::prelude::sync::SyncResolve;
use zenoh::publication::Publisher;
use zenoh::Session;

pub struct ZenohContext {
    pub session: Session,
}

impl ZenohContext {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::default();
        // config.set_mode(Some(WhatAmI::Client));
        let session = zenoh::open(config).res_sync().unwrap();
        Ok(Self { session })
    }

    pub fn put_json(&self, queue: &str, elt: &impl Serialize) -> anyhow::Result<()> {
        let publisher = self.session.declare_publisher(queue).res_sync().unwrap();
        put_json(&publisher, elt)
    }
}

pub fn put_json(publisher: &Publisher, elt: &impl Serialize) -> anyhow::Result<()> {
    let data = serde_json::to_string(elt)?;
    publisher.put(data.as_bytes()).res_sync().expect("F");
    Ok(())
}
