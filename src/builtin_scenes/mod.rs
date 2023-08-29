use bevy_rapier::plugin::RapierContext;
use bevy_rapier::rapier::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "dim2")]
mod dim2;
#[cfg(feature = "dim3")]
mod dim3;

#[cfg(feature = "dim2")]
pub use dim2::builders;
#[cfg(feature = "dim3")]
pub use dim3::builders;
use steadyum_api_types::kinematic::KinematicAnimations;

pub struct BuiltinScene {
    pub context: RapierContext,
    pub animations: HashMap<RigidBodyHandle, KinematicAnimations>,
}

impl From<RapierContext> for BuiltinScene {
    fn from(context: RapierContext) -> Self {
        Self {
            context,
            animations: Default::default(),
        }
    }
}
