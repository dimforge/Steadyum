use crate::objects::{ColdBodyObject, WarmBodyObject, WarmBodyObjectSet};
use redis::{Client, Connection};
use redis::{Cmd, Commands};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const COLD_BUCKET: &str = "steadyum-cold";
const WARM_BUCKET: &str = "steadyum-warm";

pub struct KvsContext {
    client: Client,
    connection: Connection,
}

impl KvsContext {
    pub fn new() -> anyhow::Result<Self> {
        let client = redis::Client::open("redis://127.0.0.1/")?;
        let connection = client.get_connection()?;

        Ok(Self { client, connection })
    }

    pub fn put(&mut self, key: &str, object: &impl Serialize) -> anyhow::Result<()> {
        let data = bincode::serialize(object)?;
        self.connection.set(key, data)?;
        Ok(())
    }

    pub fn put_warm(&mut self, key: &str, warm_object: &impl Serialize) -> anyhow::Result<()> {
        let data = bincode::serialize(warm_object)?;
        self.connection.set(key, data)?;
        Ok(())
    }

    pub fn put_warm_object(
        &mut self,
        uuid: Uuid,
        warm_object: &WarmBodyObject,
    ) -> anyhow::Result<()> {
        let data = bincode::serialize(warm_object)?;
        self.connection.set(uuid.as_bytes(), data)?;
        Ok(())
    }

    pub fn put_cold_object(
        &mut self,
        uuid: Uuid,
        cold_object: &ColdBodyObject,
    ) -> anyhow::Result<()> {
        let data = bincode::serialize(cold_object)?;
        self.connection.set(uuid.as_bytes(), data)?;
        Ok(())
    }

    pub fn put_multiple_cold_objects(
        &mut self,
        cold_objects: &[(Uuid, ColdBodyObject)],
    ) -> anyhow::Result<()> {
        let mut pipe = redis::pipe();
        for (uuid, cold_object) in cold_objects {
            let bytes = bincode::serialize(cold_object)?;
            pipe.add_command(Cmd::set(uuid.as_bytes(), bytes));
        }
        pipe.query(&mut self.connection)?;

        Ok(())
    }

    pub fn get_with_str_key<T>(&mut self, string: &str) -> anyhow::Result<T>
    where
        for<'a> T: Deserialize<'a>,
    {
        let bytes: Vec<u8> = self.connection.get(string)?;
        Ok(bincode::deserialize(&bytes)?)
    }

    pub fn get<T>(&mut self, uuid: Uuid) -> anyhow::Result<T>
    where
        for<'a> T: Deserialize<'a>,
    {
        let bytes: Vec<u8> = self.connection.get(uuid.as_bytes())?;
        Ok(bincode::deserialize(&bytes)?)
    }

    pub fn get_warm(&mut self, uuid: Uuid) -> anyhow::Result<WarmBodyObjectSet> {
        let bytes: Vec<u8> = self.connection.get(uuid.as_bytes())?;
        Ok(bincode::deserialize(&bytes)?)
    }

    pub fn get_multiple_warm(
        &mut self,
        uuids: &[String],
    ) -> anyhow::Result<Vec<Option<WarmBodyObjectSet>>> {
        let mut pipe = redis::pipe();
        for uuid in uuids {
            pipe.add_command(Cmd::get(uuid));
        }

        let bytes: Vec<Vec<u8>> = pipe.query(&mut self.connection)?;
        let mut result = vec![];
        for bytes in bytes {
            result.push(bincode::deserialize(&bytes).ok());
        }

        Ok(result)
    }

    pub fn get_multiple_cold(
        &mut self,
        uuids: &[Uuid],
    ) -> anyhow::Result<Vec<Option<ColdBodyObject>>> {
        let mut pipe = redis::pipe();
        for uuid in uuids {
            pipe.add_command(Cmd::get(uuid.as_bytes()));
        }

        let bytes: Vec<Vec<u8>> = pipe.query(&mut self.connection)?;
        let mut result = vec![];
        for bytes in bytes {
            result.push(bincode::deserialize(&bytes).ok());
        }

        Ok(result)
    }

    pub fn get_warm_object(&mut self, uuid: Uuid) -> anyhow::Result<WarmBodyObject> {
        let bytes: Vec<u8> = self.connection.get(uuid.as_bytes())?;
        Ok(bincode::deserialize(&bytes)?)
    }

    pub fn get_cold_object(&mut self, uuid: Uuid) -> anyhow::Result<ColdBodyObject> {
        let bytes: Vec<u8> = self.connection.get(uuid.as_bytes())?;
        Ok(bincode::deserialize(&bytes)?)
    }
}
