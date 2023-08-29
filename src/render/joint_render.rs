use crate::render::{ColliderOutlineRender, JointRender};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier::geometry::Collider;
use bevy_rapier::plugin::{RapierConfiguration, RapierContext};
use bevy_rapier::prelude::shape_views::ColliderView;
use bevy_rapier::prelude::{RapierImpulseJointHandle, RapierMultibodyJointHandle};
use bevy_rapier::rapier::dynamics::GenericJoint;
use bevy_rapier::rapier::math::{Point, Real, Vector};

pub fn render_joints(
    mut lines: ResMut<DebugLines>,
    context: Res<RapierContext>,
    impulse_joint_render: Query<(&JointRender, &RapierImpulseJointHandle)>,
    multibody_joint_render: Query<(&JointRender, &RapierMultibodyJointHandle)>,
) {
    let mut render_joint = |body1, body2, data: &GenericJoint, render: &JointRender| {
        if let (Some(rb1), Some(rb2)) = (context.bodies.get(body1), context.bodies.get(body2)) {
            let frame1 = rb1.position() * data.local_frame1;
            let frame2 = rb2.position() * data.local_frame2;

            #[cfg(feature = "dim2")]
            {
                let a = rb1.translation().push(0.0);
                let b = frame1.translation.vector.push(0.0);
                let c = frame2.translation.vector.push(0.0);
                let d = rb2.translation().push(0.0);
                lines.line_colored(a.into(), b.into(), 0.0, render.anchor_color);
                lines.line_colored(b.into(), c.into(), 0.0, render.separation_color);
                lines.line_colored(c.into(), d.into(), 0.0, render.anchor_color);
            }
            #[cfg(feature = "dim3")]
            {
                let a = *rb1.translation();
                let b = frame1.translation.vector;
                let c = frame2.translation.vector;
                let d = *rb2.translation();
                lines.line_colored(a.into(), b.into(), 0.0, render.anchor_color);
                lines.line_colored(b.into(), c.into(), 0.0, render.separation_color);
                lines.line_colored(c.into(), d.into(), 0.0, render.anchor_color);
            }
        }
    };

    for (render, handle) in impulse_joint_render.iter() {
        if let Some(joint) = context.impulse_joints.get(handle.0) {
            render_joint(joint.body1, joint.body2, &joint.data, render);
        }
    }

    for (render, handle) in multibody_joint_render.iter() {
        if let Some((multibody, id)) = context.multibody_joints.get(handle.0) {
            let link = multibody.link(id).unwrap();
            let parent = multibody.link(link.parent_id().unwrap()).unwrap();
            render_joint(
                parent.rigid_body_handle(),
                link.rigid_body_handle(),
                &link.joint.data,
                render,
            );
        }
    }
}
