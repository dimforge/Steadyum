#[cfg(feature = "dim2")]
pub extern crate rapier2d as rapier;
#[cfg(feature = "dim3")]
pub extern crate rapier3d as rapier;

pub mod kinematic;
pub mod kvs;
pub mod messages;
pub mod objects;
pub mod queries;
pub mod simulation;

pub mod region_db;
#[cfg(feature = "zenoh")]
pub mod zenoh;

pub(crate) mod array_ser;
pub mod partitionner;
