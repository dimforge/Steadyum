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

pub struct BuiltinScene {
    pub context: RapierContext,
}

impl From<RapierContext> for BuiltinScene {
    fn from(context: RapierContext) -> Self {
        Self { context }
    }
}
