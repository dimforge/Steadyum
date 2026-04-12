use crate::builtin_scenes::BuiltinScene;
use crate::physics::PhysicsState;
use crate::physics::rapier::prelude::*;
use std::collections::HashMap;

pub fn init_world() -> BuiltinScene {
    /*
     * World
     */
    let mut result = PhysicsState::default();

    /*
     * Ground
     */
    /*
     * Create the pyramids.
     */
    let body = result.bodies.insert(RigidBodyBuilder::dynamic());
    result
        .colliders
        .insert_with_parent(ColliderBuilder::ball(1.0), body, &mut result.bodies);

    BuiltinScene { state: result }
}
