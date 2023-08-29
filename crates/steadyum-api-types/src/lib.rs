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

#[cfg(feature = "zenoh")]
pub mod zenoh;
