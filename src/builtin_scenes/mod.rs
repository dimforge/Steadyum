use crate::physics::PhysicsState;

#[cfg(feature = "dim2")]
mod dim2;
#[cfg(feature = "dim3")]
mod dim3;

#[cfg(feature = "dim2")]
pub use dim2::builders;
#[cfg(feature = "dim3")]
pub use dim3::builders;

pub struct BuiltinScene {
    pub state: PhysicsState,
}

impl From<PhysicsState> for BuiltinScene {
    fn from(state: PhysicsState) -> Self {
        Self { state }
    }
}
