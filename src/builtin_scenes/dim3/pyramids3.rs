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
            // let collider = ColliderBuilder::ball(half_extents.y);
            colliders.insert_with_parent(collider, handle, bodies);
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
    let ground_size = 250.0; // 360.0;
    let ground_height = 5.0;

    let rigid_body = RigidBodyBuilder::kinematic_position_based().translation(vector![
        0.0,
        -ground_height + 25.0,
        0.0
    ]);
    let ground_handle = result.bodies.insert(rigid_body);
    let n = 10;
    // let heights = DMatrix::from_fn(n, n, |i, j| {
    //     if i == 0 || i == n - 1 || j == 0 || j == n - 1 {
    //         2.0
    //     } else {
    //         (i as f32).sin() + (j as f32).sin()
    //     }
    // });
    // let collider = ColliderBuilder::heightfield(
    //     heights,
    //     vector![ground_size * 2.0, ground_height, ground_size * 2.0],
    // );
    let collider = ColliderBuilder::cuboid(ground_size, ground_height, ground_size);
    result
        .colliders
        .insert_with_parent(collider, ground_handle, &mut result.bodies);

    let ground_animation = KinematicAnimations {
        linear: None,
        angular: Some(KinematicCurve {
            control_points: vec![vector![0.0, 0.0, 0.0], vector![0.0, 100.0, 0.0]],
            t0: 0.0,
            total_time: 1600.0,
            loop_back: true,
        }),
    };
    animations.insert(ground_handle, ground_animation);

    /*
     * Create the pyramids.
     */
    let num_basis = 6;
    let num_z = 10; // 15;
    let num_x = 25; // 30;
    let shift_y = ground_height + 25.0;

    for i in 0..num_x {
        for j in 0..num_z {
            let x = (i as f32 - num_x as f32 / 2.0) * 14.0;
            let z = (j as f32 - num_z as f32 / 2.0) * 30.0;
            create_wall(
                &mut result.bodies,
                &mut result.colliders,
                vector![x, shift_y, z],
                num_basis,
                vector![0.5, 0.5, 1.0],
            );
        }
    }

    BuiltinScene {
        context: result,
        animations,
    }
}
