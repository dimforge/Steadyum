use crate::control::KinematicAnimationsComponent;
use crate::operation::{Operation, Operations};
use crate::storage::{ExistsInDb, HandleOrUuid};
use crate::styling::ColorGenerator;
use crate::utils::{ColliderBundle, ColliderRenderBundle, RigidBodyBundle};
use bevy::prelude::*;
use bevy_rapier::prelude::*;
use std::collections::HashMap;

pub fn import_scene(
    mut commands: Commands,
    operations: Res<Operations>,
    mut colors: ResMut<ColorGenerator>,
) {
    for op in operations.iter() {
        if let Operation::ImportScene(scene) = op {
            let mut body2joints = HashMap::new();
            for (b1, b2, j) in &scene.impulse_joints {
                body2joints.entry(*b1).or_insert(vec![]).push((*b2, *j));
            }

            let mut handle2entity = HashMap::new();

            for (handle, cold_data, warm_data) in &scene.objects {
                #[cfg(feature = "dim2")]
                let transform = Transform::from_translation(Vec3::new(
                    warm_data.position.translation.x,
                    warm_data.position.translation.y,
                    0.0,
                ))
                .with_rotation(Quat::from_rotation_z(warm_data.position.rotation.angle()));
                #[cfg(feature = "dim3")]
                let transform = Transform::from_translation(warm_data.position.translation.into())
                    .with_rotation(warm_data.position.rotation.into());
                let collider_bundle = ColliderBundle {
                    collider: Collider::from(cold_data.shape.clone()),
                    ..ColliderBundle::default()
                };
                let collider_render_bundle = ColliderRenderBundle::new(&mut colors);
                let rigid_body_bundle = RigidBodyBundle {
                    rigid_body: cold_data.body_type.into(),
                    velocity: Velocity {
                        linvel: warm_data.linvel.into(),
                        angvel: warm_data.angvel.into(),
                    },
                    ..RigidBodyBundle::default()
                };

                let mut cmds = commands.spawn(rigid_body_bundle);
                cmds.insert(TransformBundle::from_transform(transform))
                    .insert(collider_bundle)
                    .insert(collider_render_bundle)
                    .insert(Visibility::default())
                    .insert(ComputedVisibility::default());

                if let HandleOrUuid::Uuid(uuid) = handle {
                    cmds.insert(ExistsInDb { uuid: *uuid });
                }

                if cold_data.animations.linear.is_some() || cold_data.animations.angular.is_some() {
                    cmds.insert(KinematicAnimationsComponent(cold_data.animations.clone()));
                }

                handle2entity.insert(*handle, cmds.id());
            }

            for (b1, b2, j) in &scene.impulse_joints {
                if let (Some(entity1), Some(entity2)) =
                    (handle2entity.get(b1), handle2entity.get(b2))
                {
                    commands.entity(*entity2).with_children(|cmd| {
                        cmd.spawn(ImpulseJoint::new(
                            *entity1,
                            bevy_rapier::dynamics::GenericJoint { raw: *j },
                        ));
                    });
                }
            }
        }
    }
}
