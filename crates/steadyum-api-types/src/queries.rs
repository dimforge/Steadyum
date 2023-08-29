use rapier::geometry::Ray;
use rapier::math::{Point, Real, Vector};
use uuid::Uuid;

#[derive(Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct RayCastQuery {
    pub ray: Ray,
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct RayCastResponse {
    pub hit: Option<Uuid>,
    pub toi: f32,
}
