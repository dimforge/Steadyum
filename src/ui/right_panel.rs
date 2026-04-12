use crate::cli::CliArgs;
use crate::control::{CharacterController, CharacterControlOptions};
use crate::physics::{ColHandle, InteractionGroups, InteractionTestMode, PhysicsState, RbHandle, Vect};
use crate::selection::Selection;
use bevy::prelude::*;
use bevy::window::Window;
use bevy_egui::{egui, EguiContexts};

use crate::physics::rapier::prelude::{Group, LockedAxes, RigidBodyType};

use super::{OpenObjectTab, UiState};

pub(super) fn ui(
    commands: &mut Commands,
    _window: &Window,
    cli: &CliArgs,
    ui_context: &mut EguiContexts,
    ui_state: &mut UiState,
    physics: &mut PhysicsState,
    rb_query: &mut Query<(Entity, &RbHandle)>,
    col_query: &mut Query<(Entity, &ColHandle)>,
    character_controllers: &mut Query<(
        &mut CharacterController,
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
        .show(ui_context.ctx_mut().expect("EguiContext"), |ui| {
            scene_explorer(commands, ui, physics, rb_query, col_query, selections, visibility);
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
                    physics,
                    rb_query,
                    col_query,
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
    physics: &mut PhysicsState,
    rb_query: &mut Query<(Entity, &RbHandle)>,
    _col_query: &mut Query<(Entity, &ColHandle)>,
    selections: &mut Query<(Entity, &mut Selection)>,
    visibility: &mut Query<(Entity, &mut Visibility)>,
) {
    ui.heading("Scene explorer");

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for (entity, rb_handle) in rb_query.iter() {
                let _body = match physics.bodies.get(rb_handle.0) {
                    Some(b) => b,
                    None => continue,
                };

                let is_selected = selections
                    .get(entity)
                    .map(|sel| sel.1.selected())
                    .unwrap_or(false);
                let _ = is_selected; // suppress unused warning

                ui.horizontal(|ui| {
                    egui::containers::CollapsingHeader::new(format!("Body {:?}", entity)).show(
                        ui,
                        |ui| {
                            if physics.entity2collider.contains_key(&entity) {
                                ui.label(format!("Collider {:?}", entity));
                            }
                        },
                    );

                    let is_visible = visibility
                        .get(entity)
                        .map(|v| v.1 != Visibility::Hidden)
                        .unwrap_or(true);
                    let visibility_icon = if is_visible { "🌑" } else { "🌕" };
                    if ui.button(visibility_icon).clicked() {
                        commands.entity(entity).insert(if is_visible {
                            Visibility::Hidden
                        } else {
                            Visibility::Visible
                        });

                        if is_visible {
                            // We hide the object, so it can't remain selected.
                            commands.entity(entity).remove::<Selection>();
                        }
                    }
                    if ui.button("❌").clicked() {
                        commands.entity(entity).despawn();
                    }
                });
            }
        });
}

fn selection_inspector(
    commands: &mut Commands,
    ui: &mut egui::Ui,
    physics: &mut PhysicsState,
    rb_query: &mut Query<(Entity, &RbHandle)>,
    col_query: &mut Query<(Entity, &ColHandle)>,
    character_controllers: &mut Query<(
        &mut CharacterController,
        &mut CharacterControlOptions,
    )>,
    selections: &mut Query<(Entity, &mut Selection)>,
    transforms: &mut Query<(Entity, &mut Transform)>,
) {
    let mut selected_any = false;
    for (entity, selected) in selections.iter() {
        if selected.selected() {
            selected_any = true;

            // --- Rigid body properties ---
            if let Ok((_entity, rb_handle)) = rb_query.get(entity) {
                let rb_h = rb_handle.0;

                if let Some(body) = physics.bodies.get(rb_h) {
                    let mut body_type = body.body_type();
                    let is_enabled = body.is_enabled();
                    let linvel = body.linvel();
                    let angvel = body.angvel();
                    let locked = body.locked_axes();
                    let is_sleeping = body.is_sleeping();
                    let ccd_enabled = body.is_ccd_enabled();
                    let mass = body.mass();

                    ui.horizontal(|ui| {
                        ui.label("Rigid-body type: ");
                        egui::ComboBox::from_id_salt("Rigid-body type")
                            .selected_text(format!("{:?}", body_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut body_type,
                                    RigidBodyType::Dynamic,
                                    "Dynamic",
                                );
                                ui.selectable_value(
                                    &mut body_type,
                                    RigidBodyType::Fixed,
                                    "Fixed",
                                );
                                ui.selectable_value(
                                    &mut body_type,
                                    RigidBodyType::KinematicPositionBased,
                                    "KinematicPositionBased",
                                );
                                ui.selectable_value(
                                    &mut body_type,
                                    RigidBodyType::KinematicVelocityBased,
                                    "KinematicVelocityBased",
                                );
                            });
                    });

                    // Apply body type change if the user selected a different one.
                    if let Some(body_mut) = physics.bodies.get_mut(rb_h) {
                        if body_mut.body_type() != body_type {
                            body_mut.set_body_type(body_type, true);
                        }
                    }

                    egui::Grid::new("Rigid-body props").show(ui, |ui| {
                        let mut enabled = is_enabled;
                        if ui.checkbox(&mut enabled, "Enabled").changed() {
                            if let Some(body_mut) = physics.bodies.get_mut(rb_h) {
                                body_mut.set_enabled(enabled);
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

                        if body_type != RigidBodyType::Fixed {
                            let mut new_linvel = linvel;
                            let mut new_angvel = angvel;

                            {
                                ui.label("Lin. velocity: ");
                                ui.add(egui::DragValue::new(&mut new_linvel.x));
                                ui.add(egui::DragValue::new(&mut new_linvel.y));
                                #[cfg(feature = "dim3")]
                                {
                                    ui.add(egui::DragValue::new(&mut new_linvel.z));
                                }
                                if ui.button("clear").clicked() {
                                    new_linvel = Vect::ZERO;
                                }
                            }
                            ui.end_row();

                            #[cfg(feature = "dim2")]
                            {
                                ui.label("Ang. velocity: ");
                                ui.add(egui::DragValue::new(&mut new_angvel));
                                if ui.button("clear").clicked() {
                                    new_angvel = 0.0;
                                }
                            }
                            #[cfg(feature = "dim3")]
                            {
                                ui.label("Ang. velocity: ");
                                ui.add(egui::DragValue::new(&mut new_angvel.x));
                                ui.add(egui::DragValue::new(&mut new_angvel.y));
                                ui.add(egui::DragValue::new(&mut new_angvel.z));
                                if ui.button("clear").clicked() {
                                    new_angvel = Vect::ZERO;
                                }
                            }
                            ui.end_row();

                            // Apply velocity changes.
                            if let Some(body_mut) = physics.bodies.get_mut(rb_h) {
                                if new_linvel != linvel {
                                    body_mut.set_linvel(new_linvel, true);
                                }
                                if new_angvel != angvel {
                                    body_mut.set_angvel(new_angvel, true);
                                }
                            }
                        }

                        // Locked axes
                        {
                            let mut new_locked = locked;

                            {
                                let mut x = new_locked.contains(LockedAxes::TRANSLATION_LOCKED_X);
                                let mut y = new_locked.contains(LockedAxes::TRANSLATION_LOCKED_Y);
                                #[cfg(feature = "dim3")]
                                let mut z = new_locked.contains(LockedAxes::TRANSLATION_LOCKED_Z);
                                ui.label("Lin. lock: ");
                                ui.checkbox(&mut x, "x");
                                ui.checkbox(&mut y, "y");

                                #[cfg(feature = "dim3")]
                                {
                                    ui.checkbox(&mut z, "z");
                                }

                                new_locked.set(LockedAxes::TRANSLATION_LOCKED_X, x);
                                new_locked.set(LockedAxes::TRANSLATION_LOCKED_Y, y);

                                #[cfg(feature = "dim3")]
                                {
                                    new_locked.set(LockedAxes::TRANSLATION_LOCKED_Z, z);
                                }
                            }
                            ui.end_row();

                            #[cfg(feature = "dim2")]
                            {
                                let mut x = new_locked.contains(LockedAxes::ROTATION_LOCKED);
                                ui.label("Ang. lock: ");
                                ui.checkbox(&mut x, "θ");
                                new_locked.set(LockedAxes::ROTATION_LOCKED, x);
                            }

                            #[cfg(feature = "dim3")]
                            {
                                let mut x = new_locked.contains(LockedAxes::ROTATION_LOCKED_X);
                                let mut y = new_locked.contains(LockedAxes::ROTATION_LOCKED_Y);
                                let mut z = new_locked.contains(LockedAxes::ROTATION_LOCKED_Z);
                                ui.label("Ang. lock: ");
                                ui.checkbox(&mut x, "θx");
                                ui.checkbox(&mut y, "θy");
                                ui.checkbox(&mut z, "θz");
                                new_locked.set(LockedAxes::ROTATION_LOCKED_X, x);
                                new_locked.set(LockedAxes::ROTATION_LOCKED_Y, y);
                                new_locked.set(LockedAxes::ROTATION_LOCKED_Z, z);
                            }
                            ui.end_row();

                            if new_locked != locked {
                                if let Some(body_mut) = physics.bodies.get_mut(rb_h) {
                                    body_mut.set_locked_axes(new_locked, true);
                                }
                            }
                        }

                        // Sleeping
                        {
                            let mut sleeping = is_sleeping;
                            ui.label("Sleeping: ");
                            if ui.checkbox(&mut sleeping, "").changed() {
                                if let Some(body_mut) = physics.bodies.get_mut(rb_h) {
                                    if sleeping {
                                        body_mut.sleep();
                                    } else {
                                        body_mut.wake_up(true);
                                    }
                                }
                            }
                            ui.end_row();
                        }

                        // Mass
                        {
                            ui.label("Mass");
                            let mut mass_val = mass;
                            ui.add(egui::DragValue::new(&mut mass_val));
                            // TODO: apply the modification if modified.
                            ui.end_row();
                        }
                    });

                    // CCD
                    {
                        let mut ccd = ccd_enabled;
                        ui.horizontal(|ui| {
                            ui.label("CCD: ");
                            if ui.checkbox(&mut ccd, "").changed() {
                                if let Some(body_mut) = physics.bodies.get_mut(rb_h) {
                                    body_mut.enable_ccd(ccd);
                                }
                            }
                        });
                    }
                }
            }

            // --- Collider properties ---
            if let Ok((_entity, col_handle)) = col_query.get(entity) {
                let col_h = col_handle.0;

                if let Some(collider) = physics.colliders.get(col_h) {
                    let coll_groups = collider.collision_groups();

                    egui::Grid::new("Collider props").show(ui, |ui| {
                        const BITS: usize = 4;
                        let mut gbits = [false; BITS];
                        let mut fbits = [false; BITS];

                        for k in 0..BITS {
                            if coll_groups.memberships.bits() & (1 << k) != 0 {
                                gbits[k] = true;
                            }
                            if coll_groups.filter.bits() & (1 << k) != 0 {
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

                        let mut new_memberships: u32 = 0;
                        let mut new_filters: u32 = 0;

                        for k in 0..BITS {
                            if gbits[k] {
                                new_memberships |= 1 << k;
                            }
                            if fbits[k] {
                                new_filters |= 1 << k;
                            }
                        }

                        let new_groups = InteractionGroups::new(
                            Group::from_bits(new_memberships).unwrap_or(Group::NONE),
                            Group::from_bits(new_filters).unwrap_or(Group::NONE),
                            InteractionTestMode::And,
                        );

                        if coll_groups.memberships != new_groups.memberships
                            || coll_groups.filter != new_groups.filter
                        {
                            if let Some(collider_mut) = physics.colliders.get_mut(col_h) {
                                collider_mut.set_collision_groups(new_groups);
                            }
                        }
                    });
                }
            }

            ui.separator();
            egui::Grid::new("Character controller props").show(ui, |ui| {
                if let Ok((mut controller, mut options)) = character_controllers.get_mut(entity) {
                    ui.label("Character controller: ");
                    ui.checkbox(&mut options.enabled, "");
                    ui.end_row();

                    if options.enabled {
                        ui.label("Gravity scale:");
                        ui.add(
                            egui::DragValue::new(&mut options.gravity_scale)
                                .range(0.0..=20.0)
                                .speed(1.0),
                        );
                        ui.end_row();

                        ui.label("Offset");
                        ui.add(
                            egui::DragValue::new(&mut controller.offset)
                                .range(0.001..=10.0)
                                .speed(0.01),
                        );
                        ui.end_row();

                        ui.label("Max slope climb angle: ");
                        ui.drag_angle(&mut controller.max_slope_climb_angle);
                        ui.end_row();

                        ui.label("Min slope slide angle: ");
                        ui.drag_angle(&mut controller.min_slope_slide_angle);
                        ui.end_row();

                        if let Some(snap_height) = &mut controller.snap_to_ground {
                            ui.label("Max snap-to-ground height");
                            ui.add(
                                egui::DragValue::new(snap_height)
                                    .range(0.0..=10.0)
                                    .speed(0.01),
                            );
                            ui.end_row();
                        }
                    }
                } else {
                    let mut has_character_controller = false;
                    ui.label("Character controller: ");
                    if ui.checkbox(&mut has_character_controller, "").changed() {
                        commands
                            .entity(entity)
                            .insert(CharacterController::default())
                            .insert(CharacterControlOptions::default());
                    }
                }
            });
        }
    }

    if !selected_any {
        ui.label("Select an object to see its properties here.");
    }
}
