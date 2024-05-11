use crate::render::{ColliderOutlineRender, ColliderRender};
use crate::styling::ColorGenerator;
use bevy::prelude::*;
use bevy_rapier::prelude::*;
use bevy_rapier::rapier::dynamics::RigidBody as RapierRigidBody;
use bevy_rapier::rapier::geometry::Collider as RapierCollider;

pub type RigidBodyComponentsMut<'a> = (
    Entity,
    &'a mut RigidBody,
    Option<&'a mut Velocity>,
    Option<&'a mut AdditionalMassProperties>,
    Option<&'a mut LockedAxes>,
    Option<&'a mut ExternalForce>,
    Option<&'a mut GravityScale>,
    Option<&'a mut Ccd>,
    Option<&'a mut Dominance>,
    Option<&'a mut Sleeping>,
    Option<&'a RigidBodyDisabled>,
    Option<&'a ReadMassProperties>,
);

pub type ColliderComponentsMut<'a> = (
    Entity,
    &'a mut Collider,
    Option<&'a mut Sensor>,
    Option<&'a mut ColliderMassProperties>,
    Option<&'a mut CollisionGroups>,
    Option<&'a ColliderDisabled>,
);

#[derive(Clone, Bundle, Default)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub mass_properties: ColliderMassProperties,
    pub collision_groups: CollisionGroups,
}

impl ColliderBundle {
    pub fn new(collider: Collider) -> Self {
        Self {
            collider,
            mass_properties: Default::default(),
            collision_groups: Default::default(),
        }
    }
}

impl<'a> From<&'a RapierCollider> for ColliderBundle {
    fn from(value: &'a RapierCollider) -> Self {
        Self {
            collider: Collider::from(value.shared_shape().clone()),
            mass_properties: Default::default(),  // FIXME
            collision_groups: Default::default(), // FIXME
        }
    }
}

#[derive(Default, Bundle)]
pub struct ColliderRenderBundle {
    pub render: ColliderRender,
    pub render_outline: ColliderOutlineRender,
    pub visibility: VisibilityBundle,
}

impl Clone for ColliderRenderBundle {
    fn clone(&self) -> Self {
        Self {
            render: self.render.clone(),
            render_outline: self.render_outline.clone(),
            visibility: VisibilityBundle {
                visibility: self.visibility.visibility.clone(),
                inherited_visibility: self.visibility.inherited_visibility.clone(),
                view_visibility: self.visibility.view_visibility.clone(),
            },
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
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, Default, Bundle)]
pub struct RigidBodyBundle {
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub additional_mass_properties: AdditionalMassProperties,
    pub mass_properties: ReadMassProperties,
    pub locked_axes: LockedAxes,
    pub forces: ExternalForce,
    pub gravity_scale: GravityScale,
    pub ccd: Ccd,
    pub dominance: Dominance,
    pub sleeping: Sleeping,
    pub damping: Damping,
}

impl RigidBodyBundle {
    pub fn dynamic() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            ..Default::default()
        }
    }

    pub fn fixed() -> Self {
        Self {
            rigid_body: RigidBody::Fixed,
            ..Default::default()
        }
    }

    pub fn kinematic_position_based() -> Self {
        Self {
            rigid_body: RigidBody::KinematicPositionBased,
            ..Default::default()
        }
    }

    pub fn kinematic_velocity_based() -> Self {
        Self {
            rigid_body: RigidBody::KinematicVelocityBased,
            ..Default::default()
        }
    }
}

impl<'a> From<&'a RapierRigidBody> for RigidBodyBundle {
    fn from(value: &'a RapierRigidBody) -> Self {
        Self {
            rigid_body: value.body_type().into(),
            velocity: Velocity {
                linvel: (*value.linvel()).into(),
                #[cfg(feature = "dim2")]
                angvel: value.angvel(),
                #[cfg(feature = "dim3")]
                angvel: (*value.angvel()).into(),
            },
            // additional_mass_properties: AdditionalMassProperties,
            // mass_properties: ReadMassProperties,
            // locked_axes: LockedAxes,
            // forces: ExternalForce,
            // gravity_scale: GravityScale,
            // ccd: Ccd,
            // dominance: Dominance,
            // sleeping: Sleeping,
            // damping: Damping,
            ..Default::default()
        }
    }
}
