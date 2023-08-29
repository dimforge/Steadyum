use crate::builtin_scenes::BuiltinScene;
use bevy_rapier::prelude::RapierContext;
use bevy_rapier3d::rapier::prelude::*;
use std::collections::HashMap;
use steadyum_api_types::kinematic::{KinematicAnimations, KinematicCurve};

pub fn init_world() -> BuiltinScene {
    /*
     * World
     */
    let mut result = RapierContext::default();
    let mut animations = HashMap::default();

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

    BuiltinScene {
        context: result,
        animations,
    }
}
