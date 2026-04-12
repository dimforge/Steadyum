use crate::render::JointRender;
use bevy::prelude::*;
use crate::physics::{PhysicsState, JointHandle, GenericJoint};

pub fn render_joints(
    mut gizmos: Gizmos,
    physics: Res<PhysicsState>,
    impulse_joint_render: Query<(&JointRender, &JointHandle)>,
) {
    let render_joint = |gizmos: &mut Gizmos, body1, body2, data: &GenericJoint, render: &JointRender| {
        if let (Some(rb1), Some(rb2)) = (physics.bodies.get(body1), physics.bodies.get(body2)) {
            let frame1 = rb1.position() * data.local_frame1;
            let frame2 = rb2.position() * data.local_frame2;

            #[cfg(feature = "dim2")]
            {
                let a = rb1.translation().extend(0.0);
                let b = frame1.translation.extend(0.0);
                let c = frame2.translation.extend(0.0);
                let d = rb2.translation().extend(0.0);
                gizmos.line(a, b, render.anchor_color);
                gizmos.line(b, c, render.separation_color);
                gizmos.line(c, d, render.anchor_color);
            }
            #[cfg(feature = "dim3")]
            {
                let a = rb1.translation();
                let b = frame1.translation;
                let c = frame2.translation;
                let d = rb2.translation();
                gizmos.line(a, b, render.anchor_color);
                gizmos.line(b, c, render.separation_color);
                gizmos.line(c, d, render.anchor_color);
            }
        }
    };

    for (render, handle) in impulse_joint_render.iter() {
        if let Some(joint) = physics.impulse_joints.get(handle.0) {
            render_joint(&mut gizmos, joint.body1, joint.body2, &joint.data, render);
        }
    }
}
