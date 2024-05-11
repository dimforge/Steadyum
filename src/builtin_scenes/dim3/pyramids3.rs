use crate::builtin_scenes::BuiltinScene;
use bevy_rapier::prelude::RapierContext;
use bevy_rapier::rapier::dynamics::RigidBodyHandle;
use bevy_rapier::rapier::prelude::*;
use na::Vector3;
use std::collections::HashMap;

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

const GROUND_SIZE: f32 = 300.0;

fn init_platform_with_walls(result: &mut RapierContext, platform_shift: Vector3<f32>) {
    /*
     * Ground
     */
    let ground_height = 5.0;

    let rigid_body = RigidBodyBuilder::kinematic_position_based()
        .translation(platform_shift + Vector3::y() * (-ground_height + 25.0));
    let ground_handle = result.bodies.insert(rigid_body);
    let n = 10;

    let collider = ColliderBuilder::cuboid(GROUND_SIZE, ground_height, GROUND_SIZE);
    result
        .colliders
        .insert_with_parent(collider, ground_handle, &mut result.bodies);

    /*
     * Create the pyramids.
     */
    let num_basis = 7;
    let num_z = 20;
    let num_x = 20;
    let shift_y = 25.5;

    for i in 0..num_x {
        for j in 0..num_z {
            let x = (i as f32 - num_x as f32 / 2.0) * (num_basis as f32 * 2.0 + 10.0);
            let z = (j as f32 - num_z as f32 / 2.0) * (num_basis as f32 * 2.0 + 10.0);
            create_wall(
                &mut result.bodies,
                &mut result.colliders,
                platform_shift + vector![x, shift_y, z],
                num_basis,
                vector![1.0, 0.5, 1.0],
            );
        }
    }
}

pub fn init_world() -> BuiltinScene {
    /*
     * World
     */
    let mut result = RapierContext::default();

    let num = 3; // 9; // 3
    for i in 0..num {
        for j in 0..num {
            let shift = vector![
                GROUND_SIZE * 2.0 * std::f32::consts::SQRT_2 * (i as f32 - (num / 2) as f32),
                0.0,
                GROUND_SIZE * 2.0 * std::f32::consts::SQRT_2 * (j as f32 - (num / 2) as f32)
            ];
            init_platform_with_walls(&mut result, shift);
        }
    }

    BuiltinScene { context: result }
}
