use bevy::prelude::*;
use bevy_rapier::{
    geometry::Collider,
    math::{Rot, Vect},
};

#[derive(Clone, Component)]
pub struct SelectionShape {
    pub translation: Vect,
    pub rotation: Rot,
    pub shape: Collider,
}

impl SelectionShape {
    pub fn new(shape: Collider) -> Self {
        Self {
            translation: Vect::ZERO,
            #[cfg(feature = "dim2")]
            rotation: 0.0,
            #[cfg(feature = "dim3")]
            rotation: Rot::IDENTITY,
            shape,
        }
    }
}
