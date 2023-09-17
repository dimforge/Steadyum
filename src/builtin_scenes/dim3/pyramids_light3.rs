use crate::builtin_scenes::BuiltinScene;
use bevy_rapier::prelude::RapierContext;
use bevy_rapier3d::rapier::prelude::*;
use std::collections::HashMap;
use steadyum_api_types::kinematic::{KinematicAnimations, KinematicCurve};

fn create_wall(
    bodies: &mut RigidBodySet,
    colliders: &mut ColliderSet,
    offset: Vector<f32>,
    stack_height: usize,
    half_extents: Vector<f32>,
) {
    let shift = half_extents * 2.0;
    for i in 0usize..stack_height {
        for j in i..stack_height {
            let fj = j as f32;
            let fi = i as f32;
            let x = offset.x;
            let y = fi * shift.y + offset.y;
            let z = (fi * shift.z / 2.0) + (fj - fi) * shift.z + offset.z
                - stack_height as f32 * half_extents.z;

            // Build the rigid body.
            let rigid_body = RigidBodyBuilder::dynamic().translation(vector![x, y, z]);
            let handle = bodies.insert(rigid_body);
            let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z);
            colliders.insert_with_parent(collider, handle, bodies);
        }
    }
}

fn create_spherical_joints(
    bodies: &mut RigidBodySet,
    colliders: &mut ColliderSet,
    impulse_joints: &mut ImpulseJointSet,
    origin: Vector<f32>,
    num: usize,
) {
    let rad = 0.4;
    let shift = 1.0;

    let mut body_handles = Vec::new();

    for k in 0..num {
        for i in 0..num {
            let fk = k as f32;
            let fi = i as f32;

            let status = if i == 0
            /* && (k % 4 == 0 || k == num - 1) */
            {
                RigidBodyType::Fixed
            } else {
                RigidBodyType::Dynamic
            };

            let rigid_body = RigidBodyBuilder::new(status)
                .translation(origin + vector![fk * shift, 0.0, fi * shift * 2.0]);
            let child_handle = bodies.insert(rigid_body);
            let collider = ColliderBuilder::cuboid(rad, rad, rad);
            colliders.insert_with_parent(collider, child_handle, bodies);

            // Vertical joint.
            if i > 0 {
                let parent_handle = *body_handles.last().unwrap();
                let joint =
                    SphericalJointBuilder::new().local_anchor2(point![0.0, 0.0, -shift * 2.0]);

                impulse_joints.insert(parent_handle, child_handle, joint, true);
            }

            // Horizontal joint.
            if k > 0 {
                let parent_index = body_handles.len() - num;
                let parent_handle = body_handles[parent_index];
                let joint = SphericalJointBuilder::new().local_anchor2(point![-shift, 0.0, 0.0]);
                impulse_joints.insert(parent_handle, child_handle, joint, true);
            }

            body_handles.push(child_handle);
        }
    }
}

pub fn init_world() -> BuiltinScene {
    /*
     * World
     */
    let mut result = RapierContext::default();
    let mut animations = HashMap::default();

    /*
     * Ground
     */
    let ground_size = 50.0;
    let ground_height = 0.1;

    let rigid_body =
        RigidBodyBuilder::kinematic_position_based().translation(vector![0.0, -ground_height, 0.0]);
    let ground_handle = result.bodies.insert(rigid_body);
    let collider = ColliderBuilder::cuboid(ground_size, ground_height, ground_size);
    result
        .colliders
        .insert_with_parent(collider, ground_handle, &mut result.bodies);

    let ground_animation = KinematicAnimations {
        linear: None,
        angular: Some(KinematicCurve {
            control_points: vec![vector![0.0, 0.0, 0.0], vector![0.0, 100.0, 0.0]],
            t0: 0.0,
            total_time: 400.0,
            loop_back: true,
        }),
    };
    animations.insert(ground_handle, ground_animation);

    /*
     * Create the pyramids.
     */
    let num_z = 8;
    let num_x = 2;
    let shift_y = ground_height + 5.5;
    let shift_z = (num_z as f32 + 2.0) * 1.0;

    for i in 0..num_x {
        let x = i as f32 * 6.0;
        create_wall(
            &mut result.bodies,
            &mut result.colliders,
            vector![x, shift_y, -shift_z],
            num_z,
            vector![0.5, 0.5, 1.0],
        );

        create_wall(
            &mut result.bodies,
            &mut result.colliders,
            vector![x, shift_y, shift_z],
            num_z - 2,
            vector![0.5, 0.5, 1.0],
        );
    }

    let num_i = 0; // 8;
    let num_j = 0; // 8;
    for i in 0..num_i {
        for j in 0..num_j {
            create_spherical_joints(
                &mut result.bodies,
                &mut result.colliders,
                &mut result.impulse_joints,
                vector![
                    (i as f32 - num_i as f32 / 2.0) * 10.0,
                    15.0,
                    (j as f32 - num_j as f32 / 2.0) * 10.0
                ],
                4,
            );
        }
    }
    BuiltinScene {
        context: result,
        animations,
    }
}
