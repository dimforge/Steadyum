use crate::cli::CliArgs;
use crate::control::CharacterControlOptions;
use crate::selection::Selection;
use crate::utils::{ColliderComponentsMut, RigidBodyComponentsMut};
use bevy::prelude::*;
use bevy::window::Window;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier::prelude::*;

use super::{OpenObjectTab, UiState};

pub(super) fn ui(
    commands: &mut Commands,
    _window: &Window,
    cli: &CliArgs,
    ui_context: &mut EguiContexts,
    ui_state: &mut UiState,
    _physics_context: &mut RapierContext,
    _physics_config: &mut RapierConfiguration,
    bodies: &mut Query<RigidBodyComponentsMut>,
    colliders: &mut Query<ColliderComponentsMut>,
    character_controllers: &mut Query<(
        &mut KinematicCharacterController,
        &mut CharacterControlOptions,
    )>,
    selections: &mut Query<(Entity, &mut Selection)>,
    visibility: &mut Query<(Entity, &mut Visibility)>,
    transforms: &mut Query<(Entity, &mut Transform)>,
) {
    if cli.lower_graphics {
        return;
    }

    egui::SidePanel::right("side_panel")
        .default_width(300.0)
        .resizable(false)
        .show(ui_context.ctx_mut(), |ui| {
            scene_explorer(commands, ui, bodies, colliders, selections, visibility);
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut ui_state.open_object_tab,
                    OpenObjectTab::SelectionInspector,
                    OpenObjectTab::SelectionInspector.rich_text(),
                );
                ui.selectable_value(
                    &mut ui_state.open_object_tab,
                    OpenObjectTab::NewBodyInspector,
                    OpenObjectTab::NewBodyInspector.rich_text(),
                );
            });

            ui.separator();

            if ui_state.open_object_tab == OpenObjectTab::SelectionInspector {
                selection_inspector(
                    commands,
                    ui,
                    bodies,
                    colliders,
                    character_controllers,
                    selections,
                    transforms,
                );
            }
        });
}

fn scene_explorer(
    commands: &mut Commands,
    ui: &mut egui::Ui,
    bodies: &mut Query<RigidBodyComponentsMut>,
    colliders: &mut Query<ColliderComponentsMut>,
    selections: &mut Query<(Entity, &mut Selection)>,
    visibility: &mut Query<(Entity, &mut Visibility)>,
) {
    ui.heading("Scene explorer");

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for (entity, ..) in bodies.iter() {
                let is_selected = selections
                    .get(entity)
                    .map(|sel| sel.1.selected())
                    .unwrap_or(false);
                ui.horizontal(|ui| {
                    egui::containers::CollapsingHeader::new(format!("Body {:?}", entity)).show(
                        ui,
                        |ui| {
                            if let Ok(_collider) = colliders.get(entity) {
                                ui.label(format!("Collider {:?}", entity));
                            }
                        },
                    );

                    let is_visible = visibility
                        .get(entity)
                        .map(|v| v.1 == Visibility::Visible)
                        .unwrap_or(true);
                    let visibility_icon = if is_visible { "🌑" } else { "🌕" };
                    if ui.button(visibility_icon).clicked() {
                        commands.entity(entity).insert(if is_visible {
                            Visibility::Hidden
                        } else {
                            Visibility::Visible
                        });

                        if is_visible {
                            // We hide the object, so it can’t remain selected.
                            commands.entity(entity).remove::<Selection>();
                        }
                    }
                    if ui.button("❌").clicked() {
                        commands.entity(entity).despawn_recursive();
                    }
                });
            }
        });
}

fn selection_inspector(
    commands: &mut Commands,
    ui: &mut egui::Ui,
    bodies: &mut Query<RigidBodyComponentsMut>,
    colliders: &mut Query<ColliderComponentsMut>,
    character_controllers: &mut Query<(
        &mut KinematicCharacterController,
        &mut CharacterControlOptions,
    )>,
    selections: &mut Query<(Entity, &mut Selection)>,
    transforms: &mut Query<(Entity, &mut Transform)>,
) {
    let mut selected_any = false;
    for (entity, selected) in selections.iter() {
        if selected.selected() {
            selected_any = true;

            if let Ok((
                _entity,
                mut rb,
                mut vel,
                _mprops,
                mut locked_axes,
                _forces,
                _gravity_scale,
                mut ccd,
                _dominance,
                mut sleep_state,
                disabled,
                read_mass_props,
            )) = bodies.get_mut(entity)
            {
                ui.horizontal(|ui| {
                    ui.label("Rigid-body type: ");
                    egui::ComboBox::from_id_source("Rigid-body type")
                        .selected_text(format!("{:?}", *rb))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut *rb, RigidBody::Dynamic, "Dynamic");
                            ui.selectable_value(&mut *rb, RigidBody::Fixed, "Fixed");
                            ui.selectable_value(
                                &mut *rb,
                                RigidBody::KinematicPositionBased,
                                "KinematicPositionBased",
                            );
                            ui.selectable_value(
                                &mut *rb,
                                RigidBody::KinematicVelocityBased,
                                "KinematicVelocityBased",
                            );
                        });
                });

                egui::Grid::new("Rigid-body props").show(ui, |ui| {
                    let mut enabled = disabled.is_none();
                    if ui.checkbox(&mut enabled, "Enabled").changed() {
                        if enabled {
                            commands.entity(entity).remove::<RigidBodyDisabled>();
                        } else {
                            commands.entity(entity).insert(RigidBodyDisabled);
                        }
                    }
                    ui.end_row();

                    if let Ok((_, mut transform)) = transforms.get_mut(entity) {
                        {
                            ui.label("Position: ");
                            let mut translation = transform.translation;
                            let mut changed =
                                ui.add(egui::DragValue::new(&mut translation.x)).changed();
                            changed = ui.add(egui::DragValue::new(&mut translation.y)).changed()
                                || changed;

                            #[cfg(feature = "dim3")]
                            {
                                changed =
                                    ui.add(egui::DragValue::new(&mut translation.z)).changed()
                                        || changed;
                            }

                            if ui.button("clear").clicked() {
                                translation = Vec3::ZERO;
                                changed = true;
                            }

                            if changed {
                                #[cfg(feature = "dim2")]
                                {
                                    transform.translation.x = translation.x;
                                    transform.translation.y = translation.y;
                                }
                                #[cfg(feature = "dim3")]
                                {
                                    transform.translation = translation;
                                }
                            }
                            ui.end_row();
                        }

                        #[cfg(feature = "dim2")]
                        {
                            let mut angle = transform.rotation.to_scaled_axis().z;

                            ui.label("Rotation: ");
                            let mut changed = ui.drag_angle(&mut angle).changed();

                            if ui.button("clear").clicked() {
                                angle = 0.0;
                                changed = true;
                            }

                            if changed {
                                transform.rotation = Quat::from_rotation_z(angle);
                            }
                            ui.end_row();
                        }

                        #[cfg(feature = "dim3")]
                        {
                            let mut axisangle = transform.rotation.to_scaled_axis();

                            ui.label("Rotation: ");
                            let mut changed = ui.drag_angle(&mut axisangle.x).changed();
                            changed = ui.drag_angle(&mut axisangle.y).changed() || changed;
                            changed = ui.drag_angle(&mut axisangle.z).changed() || changed;

                            if ui.button("clear").clicked() {
                                axisangle = Vect::ZERO;
                                changed = true;
                            }

                            if changed {
                                transform.rotation = Quat::from_scaled_axis(axisangle);
                            }
                            ui.end_row();
                        }
                    }

                    if *rb != RigidBody::Fixed {
                        if let Some(vel) = vel.as_mut() {
                            let mut new_vel = **vel;

                            {
                                ui.label("Lin. velocity: ");
                                ui.add(egui::DragValue::new(&mut new_vel.linvel.x));
                                ui.add(egui::DragValue::new(&mut new_vel.linvel.y));
                                #[cfg(feature = "dim3")]
                                {
                                    ui.add(egui::DragValue::new(&mut new_vel.linvel.z));
                                }
                                if ui.button("clear").clicked() {
                                    new_vel.linvel = Vect::ZERO;
                                }
                            }
                            ui.end_row();

                            #[cfg(feature = "dim2")]
                            {
                                ui.label("Ang. velocity: ");
                                ui.add(egui::DragValue::new(&mut new_vel.angvel));
                                if ui.button("clear").clicked() {
                                    new_vel.angvel = 0.0;
                                }
                            }
                            #[cfg(feature = "dim3")]
                            {
                                ui.label("Ang. velocity: ");
                                ui.add(egui::DragValue::new(&mut new_vel.angvel.x));
                                ui.add(egui::DragValue::new(&mut new_vel.angvel.y));
                                ui.add(egui::DragValue::new(&mut new_vel.angvel.z));
                                if ui.button("clear").clicked() {
                                    new_vel.angvel = Vect::ZERO;
                                }
                            }
                            ui.end_row();

                            if new_vel != **vel {
                                **vel = new_vel;
                            }
                        }
                    }

                    if let Some(locked_axes) = locked_axes.as_mut() {
                        {
                            let mut x = locked_axes.contains(LockedAxes::TRANSLATION_LOCKED_X);
                            let mut y = locked_axes.contains(LockedAxes::TRANSLATION_LOCKED_Y);
                            #[cfg(feature = "dim3")]
                            let mut z = locked_axes.contains(LockedAxes::TRANSLATION_LOCKED_Z);
                            ui.label("Lin. lock: ");
                            ui.checkbox(&mut x, "x");
                            ui.checkbox(&mut y, "y");

                            #[cfg(feature = "dim3")]
                            {
                                ui.checkbox(&mut z, "z");
                            }

                            locked_axes.set(LockedAxes::TRANSLATION_LOCKED_X, x);
                            locked_axes.set(LockedAxes::TRANSLATION_LOCKED_Y, y);

                            #[cfg(feature = "dim3")]
                            {
                                locked_axes.set(LockedAxes::TRANSLATION_LOCKED_Z, z);
                            }
                        }
                        ui.end_row();

                        #[cfg(feature = "dim2")]
                        {
                            let mut x = locked_axes.contains(LockedAxes::ROTATION_LOCKED);
                            ui.label("Ang. lock: ");
                            ui.checkbox(&mut x, "θ");
                            locked_axes.set(LockedAxes::ROTATION_LOCKED, x);
                        }

                        #[cfg(feature = "dim3")]
                        {
                            let mut x = locked_axes.contains(LockedAxes::ROTATION_LOCKED_X);
                            let mut y = locked_axes.contains(LockedAxes::ROTATION_LOCKED_Y);
                            let mut z = locked_axes.contains(LockedAxes::ROTATION_LOCKED_Z);
                            ui.label("Ang. lock: ");
                            ui.checkbox(&mut x, "θx");
                            ui.checkbox(&mut y, "θy");
                            ui.checkbox(&mut z, "θz");
                            locked_axes.set(LockedAxes::ROTATION_LOCKED_X, x);
                            locked_axes.set(LockedAxes::ROTATION_LOCKED_Y, y);
                            locked_axes.set(LockedAxes::ROTATION_LOCKED_Z, z);
                        }
                        ui.end_row()
                    }

                    if let Some(sleep_state) = sleep_state.as_mut() {
                        let mut sleeping = sleep_state.sleeping;
                        let mut can_sleep = sleep_state.normalized_linear_threshold > 0.0
                            && sleep_state.angular_threshold > 0.0;

                        ui.label("Sleeping: ");
                        if ui.checkbox(&mut sleeping, "").changed() {
                            sleep_state.sleeping = sleeping;
                        }
                        ui.label("Can sleep: ");
                        if ui.checkbox(&mut can_sleep, "").changed() {
                            if can_sleep {
                                **sleep_state = Sleeping {
                                    sleeping,
                                    ..Default::default()
                                }
                            } else {
                                **sleep_state = Sleeping {
                                    sleeping,
                                    ..Sleeping::disabled()
                                }
                            }
                        }

                        ui.end_row();
                    }

                    if let Some(read_mass_props) = read_mass_props {
                        let mut mprops = *read_mass_props;
                        ui.label("Mass");
                        let mut mass = mprops.mass;
                        ui.add(egui::DragValue::new(&mut mass));
                        // TODO: apply the modification if modified.
                        ui.end_row();
                    }
                });

                if let Some(ccd) = ccd.as_mut() {
                    ui.horizontal(|ui| {
                        ui.label("CCD: ");
                        ui.checkbox(&mut ccd.enabled, "");
                    });
                }
            }

            if let Ok((_entity, _collider, _sensor, _mprops, mut coll_groups, _disabled)) =
                colliders.get_mut(entity)
            {
                egui::Grid::new("Collider props").show(ui, |ui| {
                    if let Some(coll_groups) = &mut coll_groups {
                        const BITS: usize = 4;
                        let mut gbits = [false; BITS];
                        let mut fbits = [false; BITS];

                        for k in 0..BITS {
                            if coll_groups.memberships.bits() & (1 << k) != 0 {
                                gbits[k] = true;
                            }
                            if coll_groups.filters.bits() & (1 << k) != 0 {
                                fbits[k] = true;
                            }
                        }

                        ui.label("Coll. groups:  ");
                        ui.horizontal(|ui| {
                            for bit in &mut gbits {
                                ui.checkbox(bit, "");
                            }
                        });
                        ui.end_row();

                        ui.label("Coll. filters: ");
                        ui.horizontal(|ui| {
                            for bit in &mut fbits {
                                ui.checkbox(bit, "");
                            }
                        });
                        ui.end_row();

                        let mut new_memberships = 0;
                        let mut new_filters = 0;

                        for k in 0..BITS {
                            if gbits[k] {
                                new_memberships |= 1 << k;
                            }
                            if fbits[k] {
                                new_filters |= 1 << k;
                            }
                        }

                        let new_groups = CollisionGroups {
                            memberships: Group::from_bits(new_memberships).unwrap(),
                            filters: Group::from_bits(new_filters).unwrap(),
                        };

                        if **coll_groups != new_groups {
                            **coll_groups = new_groups;
                        }
                    }
                });
            }

            ui.separator();
            egui::Grid::new("Character controller props").show(ui, |ui| {
                let character_length_val =
                    |ui: &mut egui::Ui, label: &str, value: &mut CharacterLength| {
                        match value {
                            CharacterLength::Absolute(val) => {
                                ui.label(format!("{label}"));
                                ui.add(
                                    egui::DragValue::new(val).clamp_range(0.1..=10.0).speed(0.1),
                                );
                            }
                            CharacterLength::Relative(val) => {
                                ui.label(format!("{label} (%)"));
                                let mut val_percent = *val * 100.0;
                                ui.add(
                                    egui::DragValue::new(&mut val_percent)
                                        .clamp_range(1.0..=100.0)
                                        .speed(1.0),
                                );
                                *val = val_percent / 100.0;
                            }
                        }
                        ui.end_row();
                    };

                if let Ok((mut controller, mut options)) = character_controllers.get_mut(entity) {
                    ui.label("Character controller: ");
                    ui.checkbox(&mut options.enabled, "");
                    ui.end_row();

                    if options.enabled {
                        ui.label("Gravity scale:");
                        ui.add(
                            egui::DragValue::new(&mut options.gravity_scale)
                                .clamp_range(0.0..=20.0)
                                .speed(1.0),
                        );
                        ui.end_row();

                        character_length_val(ui, "Offset", &mut controller.offset);

                        ui.label("Slide");
                        ui.checkbox(&mut controller.slide, "");
                        ui.end_row();

                        if controller.slide {
                            if let Some(autostep) = &mut controller.autostep {
                                character_length_val(
                                    ui,
                                    "Max autostep height",
                                    &mut autostep.max_height,
                                );
                                character_length_val(
                                    ui,
                                    "Min autostep width",
                                    &mut autostep.min_width,
                                );

                                ui.label("Autostep on dynamic bodies");
                                ui.checkbox(&mut autostep.include_dynamic_bodies, "");
                                ui.end_row();
                            }

                            ui.label("Max slope climb angle: ");
                            ui.drag_angle(&mut controller.max_slope_climb_angle);
                            ui.end_row();

                            ui.label("Min slope slide angle: ");
                            ui.drag_angle(&mut controller.min_slope_slide_angle);
                            ui.end_row();

                            ui.label("Move dyn. rigid-bodies");
                            ui.checkbox(&mut controller.apply_impulse_to_dynamic_bodies, "");
                            ui.end_row();

                            if let Some(snap_height) = &mut controller.snap_to_ground {
                                character_length_val(ui, "Max snap-to-ground height", snap_height);
                            }
                        }
                    }
                } else {
                    let mut has_character_controller = false;
                    ui.label("Character controller: ");
                    if ui.checkbox(&mut has_character_controller, "").changed() {
                        commands
                            .entity(entity)
                            .insert(KinematicCharacterController::default())
                            .insert(CharacterControlOptions::default())
                            .insert(KinematicCharacterControllerOutput::default());
                    }
                }
            });
        }
    }

    if !selected_any {
        ui.label("Select an object to see its properties here.");
    }
}
