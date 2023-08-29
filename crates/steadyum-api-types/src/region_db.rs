use rapier::math::DIM;
use uuid::Uuid;

pub struct RegionDocument {
    pub uuid: Uuid,
    pub region: Option<[u64; DIM]>,
}
