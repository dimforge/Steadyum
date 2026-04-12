use crate::render::{ColliderOutlineRender, ColliderRender};
use crate::styling::ColorGenerator;
use bevy::prelude::*;
use crate::physics::*;
use crate::physics::rapier::dynamics::RigidBody as RapierRigidBody;
use crate::physics::rapier::geometry::Collider as RapierCollider;

/// Data needed to construct a rapier collider.
/// Not a Bevy Bundle -- used to pass collider creation data around,
/// then inserted into PhysicsState directly.
#[derive(Clone)]
pub struct ColliderBundle {
    pub collider: Collider,
    /// Override density for the collider (None = use default).
    pub density: Option<f32>,
    pub collision_groups: Option<InteractionGroups>,
}

impl ColliderBundle {
    pub fn new(collider: Collider) -> Self {
        Self {
            collider,
            density: None,
            collision_groups: None,
        }
    }
}

impl<'a> From<&'a RapierCollider> for ColliderBundle {
    fn from(value: &'a RapierCollider) -> Self {
        Self {
            collider: ColliderBuilder::new(value.shared_shape().clone()).build(),
            density: None,  // FIXME
            collision_groups: None,  // FIXME
        }
    }
}

/// Render components for a collider entity.
/// No longer a Bundle -- just a convenient grouping of render components.
#[derive(Default)]
pub struct ColliderRenderBundle {
    pub render: ColliderRender,
    pub render_outline: ColliderOutlineRender,
}

impl Clone for ColliderRenderBundle {
    fn clone(&self) -> Self {
        Self {
            render: self.render.clone(),
            render_outline: self.render_outline.clone(),
        }
    }
}

impl ColliderRenderBundle {
    pub fn new(colors: &mut ColorGenerator) -> Self {
        let color = colors.gen_color();
        let outline_color = ColorGenerator::outline_color(color);
        Self {
            render: ColliderRender::from(color),
            render_outline: ColliderOutlineRender::new(outline_color, 0.02),
        }
    }

    /// Insert these render components (plus Visibility) into a Bevy entity.
    pub fn insert_into(self, commands: &mut EntityCommands) {
        commands.insert((
            self.render,
            self.render_outline,
            Visibility::default(),
        ));
    }
}

/// Data needed to construct a rapier rigid body.
/// Not a Bevy Bundle -- used to pass rigid body creation data around,
/// then inserted into PhysicsState directly.
#[derive(Copy, Clone)]
pub struct RigidBodyBundle {
    pub rigid_body_type: RigidBodyType,
    pub linvel: Vect,
    #[cfg(feature = "dim2")]
    pub angvel: f32,
    #[cfg(feature = "dim3")]
    pub angvel: Vect,
    pub gravity_scale: f32,
    pub ccd_enabled: bool,
    pub dominance: i8,
    pub sleeping: bool,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl Default for RigidBodyBundle {
    fn default() -> Self {
        Self {
            rigid_body_type: RigidBodyType::Dynamic,
            linvel: Vect::ZERO,
            #[cfg(feature = "dim2")]
            angvel: 0.0,
            #[cfg(feature = "dim3")]
            angvel: Vect::ZERO,
            gravity_scale: 1.0,
            ccd_enabled: false,
            dominance: 0,
            sleeping: false,
            linear_damping: 0.0,
            angular_damping: 0.0,
        }
    }
}

impl RigidBodyBundle {
    pub fn dynamic() -> Self {
        Self {
            rigid_body_type: RigidBodyType::Dynamic,
            gravity_scale: 1.0,
            ..Default::default()
        }
    }

    pub fn fixed() -> Self {
        Self {
            rigid_body_type: RigidBodyType::Fixed,
            gravity_scale: 1.0,
            ..Default::default()
        }
    }

    pub fn kinematic_position_based() -> Self {
        Self {
            rigid_body_type: RigidBodyType::KinematicPositionBased,
            gravity_scale: 1.0,
            ..Default::default()
        }
    }

    pub fn kinematic_velocity_based() -> Self {
        Self {
            rigid_body_type: RigidBodyType::KinematicVelocityBased,
            gravity_scale: 1.0,
            ..Default::default()
        }
    }

    /// Build a rapier `RigidBodyBuilder` from this bundle's data.
    pub fn into_builder(self) -> RigidBodyBuilder {
        RigidBodyBuilder::new(self.rigid_body_type)
            .linvel(self.linvel)
            .angvel(self.angvel)
            .gravity_scale(self.gravity_scale)
            .ccd_enabled(self.ccd_enabled)
            .dominance_group(self.dominance)
            .sleeping(self.sleeping)
            .linear_damping(self.linear_damping)
            .angular_damping(self.angular_damping)
    }
}

impl<'a> From<&'a RapierRigidBody> for RigidBodyBundle {
    fn from(value: &'a RapierRigidBody) -> Self {
        Self {
            rigid_body_type: value.body_type(),
            linvel: value.linvel(),
            angvel: value.angvel(),
            gravity_scale: value.gravity_scale(),
            ccd_enabled: value.is_ccd_enabled(),
            dominance: value.dominance_group(),
            sleeping: value.is_sleeping(),
            linear_damping: value.linear_damping(),
            angular_damping: value.angular_damping(),
        }
    }
}
