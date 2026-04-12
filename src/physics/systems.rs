use bevy::prelude::*;
use super::components::RbHandle;
use super::state::PhysicsState;

/// Steps the physics simulation forward by one timestep.
/// When physics is paused, steps with `dt=0` so nothing moves but broad/narrow phase
/// state stays in sync with newly added/removed/modified colliders.
pub fn physics_step(mut physics: ResMut<PhysicsState>) {
    let gravity = physics.gravity;
    let real_dt = physics.integration_parameters.dt;
    let running = physics.running;

    if !running {
        physics.integration_parameters.dt = 0.0;
    }

    let PhysicsState {
        ref mut pipeline,
        ref integration_parameters,
        ref mut island_manager,
        ref mut broad_phase,
        ref mut narrow_phase,
        ref mut bodies,
        ref mut colliders,
        ref mut impulse_joints,
        ref mut multibody_joints,
        ref mut ccd_solver,
        ref mut counters,
        ..
    } = *physics;

    pipeline.step(
        gravity,
        integration_parameters,
        island_manager,
        broad_phase,
        narrow_phase,
        bodies,
        colliders,
        impulse_joints,
        multibody_joints,
        ccd_solver,
        &(),
        &(),
    );

    // Copy counters from the pipeline.
    *counters = pipeline.counters.clone();

    // Restore the real timestep if we overrode it.
    if !running {
        physics.integration_parameters.dt = real_dt;
    }
}

/// Syncs rapier rigid body positions back to Bevy Transforms.
pub fn sync_physics_to_transforms(
    physics: Res<PhysicsState>,
    mut query: Query<(&RbHandle, &mut Transform)>,
) {
    for (rb_handle, mut transform) in query.iter_mut() {
        if let Some(body) = physics.bodies.get(rb_handle.0) {
            let pos = body.position();
            #[cfg(feature = "dim3")]
            {
                transform.translation = pos.translation;
                transform.rotation = pos.rotation.into();
            }
            #[cfg(feature = "dim2")]
            {
                transform.translation = pos.translation.extend(transform.translation.z);
                transform.rotation = bevy::math::Quat::from_rotation_z(pos.rotation.angle());
            }
        }
    }
}
