use crate::builtin_scenes::BuiltinScene;
use bevy_rapier::prelude::RapierContext;
use bevy_rapier2d::rapier::prelude::*;
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
            let x = (fi * shift.x / 2.0) + (fj - fi) * shift.x + offset.x
                - stack_height as f32 * half_extents.x;
            let y = fi * shift.y + offset.y;

            // Build the rigid body.
            let rigid_body = RigidBodyBuilder::dynamic().translation(vector![x, y]);
            let handle = bodies.insert(rigid_body);
            let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y);
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
     * Create the pyramids.
     */
    let num_basis = 7;
    let num_x = 20;
    let num_y = 10;

    for j in 0..num_y {
        let y = j as f32 * 14.0 + 2.0;

        for i in 0..num_x {
            let x = (i as f32 - num_x as f32 / 2.0) * 16.0;
            create_wall(
                &mut result.bodies,
                &mut result.colliders,
                vector![x, y + 4.0],
                num_basis,
                vector![0.5, 0.5],
            );

            let rigid_body =
                RigidBodyBuilder::kinematic_position_based().translation(vector![x, y]);
            let ground_handle = result.bodies.insert(rigid_body);
            let collider = ColliderBuilder::cuboid(8.0, 0.5);
            result
                .colliders
                .insert_with_parent(collider, ground_handle, &mut result.bodies);

            // let ground_animation = KinematicAnimations {
            //     linear: Some(KinematicCurve {
            //         control_points: vec![vector![x, y], vector![x, y + 1.0]],
            //         t0: 0.0,
            //         total_time: 16.0,
            //         loop_back: true,
            //     }),
            //     angular: None,
            // };
            // animations.insert(ground_handle, ground_animation);
        }
    }

    BuiltinScene {
        context: result,
        animations,
    }
}
