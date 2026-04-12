use bevy::prelude::*;
use crate::physics::{Vect, ImpulseJointHandle};

mod mouse;

/// Stores the joint handle (in PhysicsState) instead of an entity for the joint.
#[derive(Default, Clone, Resource)]
pub struct DragState {
    pub drag_local_point: Vect,
    pub drag_plane_point: Vect,
    pub drag_plane_normal: Vect,
    pub dragged_entity: Option<Entity>,
    /// The temporary kinematic body entity used for dragging.
    pub mouse_body: Option<Entity>,
    /// The spring joint handle in PhysicsState.impulse_joints.
    pub spring_joint: Option<ImpulseJointHandle>,
}

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DragState::default())
            .add_systems(Update, mouse::handle_drag_click)
            .add_systems(Update, mouse::handle_drag_hover);
    }
}
