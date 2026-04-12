use bevy::prelude::*;
use crate::physics::{SharedShape, Vect, Rotation};

#[derive(Clone, Component)]
pub struct SelectionShape {
    pub translation: Vect,
    pub rotation: Rotation,
    pub shape: SharedShape,
}

impl SelectionShape {
    pub fn new(shape: SharedShape) -> Self {
        Self {
            translation: Vect::ZERO,
            #[cfg(feature = "dim2")]
            rotation: Rotation::IDENTITY,
            #[cfg(feature = "dim3")]
            rotation: Rotation::IDENTITY,
            shape,
        }
    }
}
