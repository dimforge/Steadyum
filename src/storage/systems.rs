use crate::control::KinematicAnimationsComponent;
use crate::operation::{Operation, Operations};
use crate::render::ColliderRender;
use crate::selection::Selection;
use crate::storage::db::{CameraPos, DbCommand, DbContext, NewObjectCommand};
use crate::storage::plugin::ExistsInDb;
use crate::storage::{HandleOrUuid, SaveFileData};
use crate::styling::ColorGenerator;
use crate::ui::UiState;
use crate::{MainCamera, PhysicsProgress};
#[cfg(feature = "dim2")]
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::Uuid;
use bevy_rapier::dynamics::RigidBody;
use bevy_rapier::math::{Real, Vect};
use bevy_rapier::plugin::RapierContext;
use bevy_rapier::prelude::RapierColliderHandle;
use bevy_rapier::prelude::RapierImpulseJointHandle;
use bevy_rapier::prelude::RapierRigidBodyHandle;
use bevy_rapier::rapier::math::Isometry;
use bevy_rapier::utils::{iso_to_transform, transform_to_iso};
use std::collections::VecDeque;
use steadyum_api_types::messages::{
    ImpulseJointAssignment, PartitionnerMessage, PARTITIONNER_QUEUE,
};
use steadyum_api_types::objects::{ColdBodyObject, WarmBodyObject};

#[derive(Copy, Clone, Debug, Default)]
struct PositionInterpolationPoint {
    pub pos: Isometry<Real>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Component)]
pub struct PositionInterpolation {
    current: PositionInterpolationPoint,
    targets: VecDeque<PositionInterpolationPoint>,
}

impl From<Isometry<Real>> for PositionInterpolation {
    fn from(pos: Isometry<Real>) -> Self {
        Self {
            current: PositionInterpolationPoint { pos, timestamp: 0 },
            targets: VecDeque::new(),
        }
    }
}

impl PositionInterpolation {
    pub fn step(&mut self, timestamp: u64) {
        while !self.targets.is_empty() {
            if self.targets[0].timestamp <= timestamp {
                self.current = self.targets.pop_front().unwrap();
            } else {
                break;
            }
        }

        // Now, interpolate between the current pos and the target pos.
        if !self.targets.is_empty() {
            let target = &self.targets[0];
            let t = (timestamp as Real - self.current.timestamp as Real).max(0.0)
                / (target.timestamp as Real - self.current.timestamp as Real);
            self.current.pos = self.current.pos.lerp_slerp(&target.pos, t);
            self.current.timestamp = timestamp;
        }
    }

    pub fn current_pos(&self) -> Isometry<Real> {
        self.current.pos
    }

    pub fn final_pos(&self) -> &Isometry<Real> {
        self.targets
            .back()
            .map(|p| &p.pos)
            .unwrap_or(&self.current.pos)
    }

    pub fn max_known_timestep(&self) -> u64 {
        self.targets
            .back()
            .map(|p| p.timestamp)
            .unwrap_or(self.current.timestamp)
    }

    pub fn add_interpolation_point(&mut self, pos: Isometry<Real>, timestamp: u64) {
        // TODO: don’t accumulate interpolation point with equal positions, or with
        //       position that could be part of the interpolation.
        self.targets
            .push_back(PositionInterpolationPoint { pos, timestamp });
    }
}

pub fn publish_new_objects_to_kvs(
    mut commands: Commands,
    mut db: ResMut<DbContext>,
    physics: Res<RapierContext>,
    added_colliders: Query<
        (
            Entity,
            &RapierColliderHandle,
            &RapierRigidBodyHandle,
            Option<&KinematicAnimationsComponent>,
            Option<&ExistsInDb>,
        ),
        Added<RapierColliderHandle>,
    >,
    added_joints: Query<(Entity, &RapierImpulseJointHandle), Without<ExistsInDb>>,
) {
    let mut objects = vec![];
    for (entity, co_handle, rb_handle, animations, exists_in_db) in added_colliders.iter() {
        if let Some(in_db) = exists_in_db {
            db.to_monitor.insert(in_db.uuid);
            db.rb2uuid.insert(rb_handle.0, in_db.uuid);
            db.uuid2rb.insert(in_db.uuid, rb_handle.0);
            continue;
        }

        dbg!("Publishing");

        if let Some(collider) = physics.colliders.get(co_handle.0) {
            if let Some(body) = physics.bodies.get(
                collider
                    .parent()
                    .expect("Parentless colliders not supported."),
            ) {
                let mut cold_object = ColdBodyObject::from_body_collider(body, collider);

                if let Some(animations) = animations {
                    cold_object.animations = animations.0.clone();
                }

                let warm_object = WarmBodyObject::from_body(body, 0);
                let uuid = exists_in_db.map(|uuid| uuid.uuid).unwrap_or(Uuid::new_v4());
                objects.push(NewObjectCommand {
                    uuid: uuid.clone(),
                    handle: rb_handle.0,
                    cold_object,
                    warm_object,
                });
                db.to_monitor.insert(uuid.clone());
                db.rb2uuid.insert(rb_handle.0, uuid);
                commands.entity(entity).insert(ExistsInDb { uuid });
            }
        }
    }

    if !objects.is_empty() {
        db.commands_snd
            .send(DbCommand::NewObjects { objects })
            .unwrap();
    }

    let mut joints = vec![];
    for (entity, joint_handle) in added_joints.iter() {
        if let Some(joint) = physics.impulse_joints.get(joint_handle.0) {
            if !db.rb2uuid.contains_key(&joint.body1) || !db.rb2uuid.contains_key(&joint.body2) {
                continue;
            }

            let assignment = ImpulseJointAssignment {
                body1: db.rb2uuid[&joint.body1],
                body2: db.rb2uuid[&joint.body2],
                joint: joint.data,
            };
            joints.push(assignment);
            commands.entity(entity).insert(ExistsInDb {
                uuid: Uuid::new_v4(),
            });
        }
    }

    if !joints.is_empty() {
        db.commands_snd
            .send(DbCommand::NewJoints { joints })
            .unwrap();
    }
}

pub fn update_start_stop(mut db: ResMut<DbContext>, ui: Res<UiState>) {
    if db.is_running != ui.running {
        dbg!("Update start stop.");
        db.is_running = ui.running;
        db.partitionner.set_running(db.is_running).unwrap();
    }
}

pub fn write_selected_object_position_to_kvs(
    db: Res<DbContext>,
    bodies: Query<(&ExistsInDb, &Transform, &Selection), Changed<Transform>>,
) {
    for (db_entry, transform, selection) in bodies.iter() {
        if selection.selected() {
            #[cfg(feature = "dim2")]
            let position = Isometry::from((
                transform.translation.xy(),
                transform.rotation.to_scaled_axis().z,
            ));
            #[cfg(feature = "dim3")]
            let position = Isometry::from((transform.translation, transform.rotation));

            let message = PartitionnerMessage::MoveObject {
                uuid: db_entry.uuid.clone(),
                position,
            };
            dbg!("Sending new selected position");
            db.zenoh
                .lock()
                .unwrap()
                .put_json(PARTITIONNER_QUEUE, &message)
                .unwrap();
        }
    }
}

pub fn write_modified_cold_objects_to_kvs(
    mut db: ResMut<DbContext>,
    ctxt: Res<RapierContext>,
    bodies: Query<(&ExistsInDb, &RapierRigidBodyHandle, &RapierColliderHandle), Changed<RigidBody>>,
) {
    for (in_db, body, collider) in bodies.iter() {
        if let (Some(rb), Some(co)) = (ctxt.bodies.get(body.0), ctxt.colliders.get(collider.0)) {
            let cold = ColdBodyObject::from_body_collider(rb, co);
            db.kvs.put_cold_object(in_db.uuid, &cold).unwrap();

            let message = PartitionnerMessage::UpdateColdObject { uuid: in_db.uuid };
            db.zenoh
                .lock()
                .unwrap()
                .put_json(PARTITIONNER_QUEUE, &message)
                .unwrap();
        }
    }
}

pub fn read_new_objects_from_kvs(db: Res<DbContext>, mut operations: ResMut<Operations>) {
    let mut scene = SaveFileData::default();

    while let Ok(obj) = db.new_objects_rcv.try_recv() {
        scene.objects.push((
            HandleOrUuid::Uuid(obj.pos.uuid),
            obj.body,
            WarmBodyObject {
                timestamp: obj.pos.timestamp,
                position: obj.pos.position,
                linvel: na::zero(),
                angvel: na::zero(),
            },
        ))
    }

    operations.push(Operation::ImportScene(scene));
}

pub fn read_object_positions_from_kvs(
    db: Res<DbContext>,
    mut progress: ResMut<PhysicsProgress>,
    mut colors: ResMut<ColorGenerator>,
    mut bodies: Query<(
        &Transform,
        &RapierRigidBodyHandle,
        &Selection,
        &mut PositionInterpolation,
        &mut ColliderRender,
    )>,
) {
    let latest_data = db.latest_data.read().unwrap();
    let mut new_progress_limit = u64::MAX;
    let t0 = instant::Instant::now();

    for (transform, rb_handle, selection, mut interpolation, mut color) in bodies.iter_mut() {
        if let Some(data) = latest_data.get(rb_handle.0 .0) {
            if false && selection.selected() {
                #[cfg(feature = "dim2")]
                let position = Isometry::from((
                    transform.translation.xy(),
                    transform.rotation.to_scaled_axis().z,
                ));
                #[cfg(feature = "dim3")]
                let position = Isometry::from((transform.translation, transform.rotation));
                interpolation.add_interpolation_point(position, data.data.timestamp);
            } else {
                interpolation.add_interpolation_point(data.data.position, data.data.timestamp);
            }

            // new_progress_limit = new_progress_limit.min(data.data.timestamp);

            // NOTE: don’t trigger the color change detection if it didn’t change.
            let region_color = colors.gen_region_color(data.region.clone());
            if region_color != color.color {
                color.color = region_color;
            }
        }

        new_progress_limit = new_progress_limit.min(interpolation.max_known_timestep());
    }

    // dbg!(latest_data.iter().count());
    if new_progress_limit != u64::MAX {
        progress.progress_limit = progress.progress_limit.max(new_progress_limit as usize);
    }

    if t0.elapsed().as_secs_f32() > 0.01 {
        // println!("read form kvs: {}", t0.elapsed().as_secs_f32());
    }
}

pub fn add_interpolation_components(
    mut commands: Commands,
    physics: Res<RapierContext>,
    objects: Query<
        (Entity, &RapierRigidBodyHandle),
        (With<RigidBody>, Without<PositionInterpolation>),
    >,
) {
    for (entity, handle) in objects.iter() {
        if let Some(rb) = physics.bodies.get(handle.0) {
            if rb.is_dynamic() {
                let interp = PositionInterpolation::from(*rb.position());
                commands
                    .entity(entity)
                    .insert(PositionInterpolation::from(interp));
            }
        }
    }
}

pub fn step_interpolations(
    ui_state: Res<UiState>,
    progress: Res<PhysicsProgress>,
    mut objects: Query<(&mut PositionInterpolation, &mut Transform)>,
) {
    let t0 = instant::Instant::now();

    // println!(
    //     "Progress: {}/{}",
    //     progress.simulated_steps, progress.progress_limit
    // );

    for (mut interpolation, mut transform) in objects.iter_mut() {
        interpolation.step(progress.simulated_steps as u64);

        let current_pos = if ui_state.interpolation {
            iso_to_transform(&interpolation.current_pos(), 1.0)
        } else {
            iso_to_transform(interpolation.final_pos(), 1.0)
        };

        transform.translation = current_pos.translation;
        transform.rotation = current_pos.rotation;
    }

    if t0.elapsed().as_secs_f32() > 0.01 {
        println!("step interpolations: {}", t0.elapsed().as_secs_f32());
    }
}

pub fn integrate_kinematic_animations(
    progress: Res<PhysicsProgress>,
    mut objects: Query<(&mut Transform, &KinematicAnimationsComponent)>,
) {
    for (mut transform, animations) in objects.iter_mut() {
        let base = transform_to_iso(&*transform, 1.0);
        let pos = animations.0.eval(progress.simulated_time, base);
        *transform = iso_to_transform(&pos, 1.0);
    }
}

pub fn update_camera_pos(db: Res<DbContext>, camera: Query<&Transform, With<MainCamera>>) {
    #[cfg(feature = "dim3")]
    for transform in camera.iter() {
        let camera_pos = CameraPos {
            position: transform.translation,
            dir: transform.rotation * -Vect::Z,
        };
        *db.camera.write().unwrap() = camera_pos;
    }
}

// TODO: move to its own file?
pub fn update_physics_progress(
    mut progress: ResMut<PhysicsProgress>,
    context: Res<RapierContext>,
    ui_state: Res<UiState>,
) {
    if ui_state.running {
        println!(
            "sim steps: {}, limit: {}",
            progress.simulated_steps, progress.progress_limit
        );
        if progress.simulated_steps <= progress.progress_limit {
            progress.simulated_time += context.integration_parameters.dt;
            progress.simulated_steps += 1;
        }
    } else {
        progress.simulated_steps = progress.progress_limit;
        progress.simulated_time =
            context.integration_parameters.dt * progress.simulated_steps as Real;
    }
}
