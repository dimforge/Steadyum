// This tranform gizmo was extracted (and modified) from the bevy_transform_gizmo crate:
// https://github.com/ForesightMiningSoftwareCorporation/bevy_transform_gizmo.

use super::{SceneMouse, SelectableSceneObject, Selection};
use crate::parry::query;
use bevy::{ecs::schedule::ShouldRun, input::InputSystem, prelude::*, transform::TransformSystem};
use bevy_rapier::dynamics::ReadMassProperties;
use gizmo_material::{GizmoMaterial, GizmoStateMaterials};
use normalization::*;

mod gizmo_material;
mod mesh;
mod normalization;

#[derive(Default, Copy, Clone, Component)]
pub struct PickableGizmo;

#[derive(Resource)]
pub struct GizmoSystemsEnabled(pub bool);
use crate::{GizmoCamera, MainCamera, GIZMO_LAYER};
pub use normalization::Ui3dNormalization;

#[cfg(feature = "dim2")]
use crate::OrbitCamera;

#[derive(Clone, Hash, PartialEq, Eq, Debug, RunCriteriaLabel)]
pub struct GizmoSystemsEnabledCriteria;

fn plugin_enabled(enabled: Res<GizmoSystemsEnabled>) -> ShouldRun {
    if enabled.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum TransformGizmoSystem {
    UpdateSettings,
    Place,
    Hover,
    Grab,
    Drag,
}

#[derive(Debug)]
pub struct TransformGizmoEvent {
    pub from: GlobalTransform,
    pub to: GlobalTransform,
    pub interaction: TransformGizmoInteraction,
}

#[derive(Component)]
pub struct GizmoTransformable;

#[derive(Resource)]
pub struct GizmoSettings {
    /// Rotation to apply to the gizmo when it is placed. Used to align the gizmo to a different
    /// coordinate system.
    pub alignment_rotation: Quat,
}

#[derive(Default)]
pub struct TransformGizmoPlugin {
    // Rotation to apply to the gizmo when it is placed. Used to align the gizmo to a different
    // coordinate system.
    pub alignment_rotation: Quat,
}

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            gizmo_material::GIZMO_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("./assets/gizmo_material.wgsl")),
        );
        let alignment_rotation = self.alignment_rotation;
        app.insert_resource(GizmoSettings { alignment_rotation })
            .insert_resource(GizmoSystemsEnabled(true))
            .add_plugin(MaterialPlugin::<GizmoMaterial>::default())
            .add_event::<TransformGizmoEvent>()
            .add_plugin(Ui3dNormalization)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(plugin_enabled.label(GizmoSystemsEnabledCriteria))
                    .with_system(
                        hover_gizmo
                            .label(TransformGizmoSystem::Hover)
                            .after(TransformGizmoSystem::UpdateSettings)
                            .after(InputSystem), // Needed otherwise we may miss some `just_released/just_pressed` state changes.
                    )
                    .with_system(
                        grab_gizmo
                            .label(TransformGizmoSystem::Grab)
                            .after(TransformGizmoSystem::Hover),
                    ),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .with_run_criteria(plugin_enabled.label(GizmoSystemsEnabledCriteria))
                    .with_system(update_gizmo_alignment.label(TransformGizmoSystem::UpdateSettings))
                    .with_system(
                        drag_gizmo
                            .label(TransformGizmoSystem::Drag)
                            .before(TransformSystem::TransformPropagate),
                    )
                    .with_system(
                        place_gizmo
                            .label(TransformGizmoSystem::Place)
                            .before(TransformSystem::TransformPropagate)
                            .after(TransformGizmoSystem::Drag),
                    ),
            )
            .add_startup_system(mesh::build_gizmo)
            .add_startup_system_to_stage(StartupStage::PostStartup, place_gizmo)
            .add_system(sync_gizmo_camera);
    }
}

#[derive(Bundle)]
pub struct TransformGizmoBundle {
    gizmo: TransformGizmo,
    interaction: Interaction,
    transform: Transform,
    global_transform: GlobalTransform,
    visible: Visibility,
    computed_visibility: ComputedVisibility,
    normalize: Normalize3d,
}

impl Default for TransformGizmoBundle {
    fn default() -> Self {
        TransformGizmoBundle {
            transform: Transform::from_translation(Vec3::splat(f32::MIN)),
            interaction: Interaction::None,
            visible: Visibility { is_visible: false },
            computed_visibility: ComputedVisibility::default(),
            gizmo: TransformGizmo::default(),
            global_transform: GlobalTransform::default(),
            normalize: Normalize3d::new(1.5, 150.0),
        }
    }
}

#[derive(Default, PartialEq, Component)]
pub struct TransformGizmo {
    pub current_interaction: Option<TransformGizmoInteraction>,
    // Point in space where mouse-gizmo interaction started (on mouse down), used to compare how
    // much total dragging has occurred without accumulating error across frames.
    drag_start: Option<Vec3>,
    origin_drag_start: Option<Vec3>,
    // Initial transform of the gizmo
    initial_transform: Option<GlobalTransform>,
}

impl TransformGizmo {
    /// Get the gizmo's drag direction.
    pub fn current_interaction(&self) -> Option<TransformGizmoInteraction> {
        self.current_interaction
    }
}

/// Marks the current active gizmo interaction
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub enum TransformGizmoInteraction {
    TranslateAxis { original: Vec3, axis: Vec3 },
    RotateAxis { original: Vec3, axis: Vec3 },
    ScaleAxis { original: Vec3, axis: Vec3 },
}

#[derive(Component)]
struct InitialTransform {
    transform: GlobalTransform,
}

/// Updates the position of the gizmo and selected meshes while the gizmo is being dragged.
#[allow(clippy::type_complexity)]
fn drag_gizmo(
    scene_mouse: Res<SceneMouse>,
    mut gizmo_mut: Query<&mut TransformGizmo>,
    mut transform_queries: ParamSet<(
        Query<(
            &Selection,
            &mut Transform,
            &InitialTransform,
            Option<&ReadMassProperties>,
        )>,
        Query<(&GlobalTransform, &Interaction), With<TransformGizmo>>,
    )>,
) {
    #[cfg(feature = "dim2")]
    if let Some(point) = scene_mouse.point {
        use bevy::math::Vec3Swizzles;

        let gizmo_transform =
            if let Ok((transform, &Interaction::Clicked)) = transform_queries.p1().get_single() {
                transform.to_owned().compute_transform()
            } else {
                return;
            };

        let mut gizmo = if let Ok(g) = gizmo_mut.get_single_mut() {
            g
        } else {
            error!("Number of transform gizmos is != 1");
            return;
        };

        let gizmo_origin = match gizmo.origin_drag_start {
            Some(origin) => origin,
            None => {
                let origin = gizmo_transform.translation;
                gizmo.origin_drag_start = Some(origin);
                origin
            }
        };

        if let Some(interaction) = gizmo.current_interaction {
            // dbg!(interaction);
            if gizmo.initial_transform.is_none() {
                gizmo.initial_transform = Some(gizmo_transform.into());
            }
            match interaction {
                TransformGizmoInteraction::TranslateAxis { original: _, axis } => {
                    let drag_start = match &gizmo.drag_start {
                        Some(drag_start) => *drag_start,
                        None => {
                            gizmo.drag_start = Some(Vec3::new(point.x, point.y, 0.0));
                            return;
                        }
                    };

                    let translation = axis * (point - drag_start.xy()).dot(axis.xy());

                    transform_queries
                        .p0()
                        .iter_mut()
                        .filter(|(s, _t, _i, _mprops)| s.selected())
                        .for_each(|(_s, mut t, i, _mprops)| {
                            let i = i.transform.compute_transform();
                            *t = Transform {
                                translation: i.translation + translation,
                                rotation: i.rotation,
                                scale: i.scale,
                            }
                        });
                }
                TransformGizmoInteraction::RotateAxis { .. } => {
                    let drag_start = match &gizmo.drag_start {
                        Some(drag_start) => *drag_start,
                        None => {
                            gizmo.drag_start = Some(Vec3::new(point.x, point.y, 0.0));
                            return;
                        }
                    };

                    let delta_angle = (drag_start - gizmo_origin)
                        .xy()
                        .angle_between(point - gizmo_origin.xy());

                    transform_queries
                        .p0()
                        .iter_mut()
                        .filter(|(s, _t, _i, _mprops)| s.selected())
                        .for_each(|(_s, mut t, i, _mprops)| {
                            let i = i.transform.compute_transform();
                            *t = Transform {
                                translation: i.translation,
                                rotation: Quat::from_rotation_z(delta_angle) * i.rotation,
                                scale: i.scale,
                            }
                        });
                }
                TransformGizmoInteraction::ScaleAxis {
                    original: _,
                    axis: _,
                } => (),
            }
        }
    }
    #[cfg(feature = "dim3")]
    if let Some((ray_start, ray_dir)) = scene_mouse.ray {
        // Gizmo handle should project mouse motion onto the axis of the handle. Perpendicular motion
        // should have no effect on the handle. We can do this by projecting the vector from the handle
        // click point to mouse's current position, onto the axis of the direction we are dragging. See
        // the wiki article for details: https://en.wikipedia.org/wiki/Vector_projection
        let gizmo_transform =
            if let Ok((transform, &Interaction::Clicked)) = transform_queries.p1().get_single() {
                transform.to_owned().compute_transform()
            } else {
                return;
            };

        // println!("Interacting");

        let mut gizmo = if let Ok(g) = gizmo_mut.get_single_mut() {
            g
        } else {
            error!("Number of transform gizmos is != 1");
            return;
        };

        let gizmo_origin = match gizmo.origin_drag_start {
            Some(origin) => origin,
            None => {
                let origin = gizmo_transform.translation;
                gizmo.origin_drag_start = Some(origin);
                origin
            }
        };
        if let Some(interaction) = gizmo.current_interaction {
            // dbg!(interaction);
            if gizmo.initial_transform.is_none() {
                gizmo.initial_transform = Some(gizmo_transform.into());
            }
            match interaction {
                TransformGizmoInteraction::TranslateAxis { original: _, axis } => {
                    let vertical_vector = ray_dir.cross(axis).normalize();
                    let plane_normal = axis.cross(vertical_vector).normalize();
                    let plane_origin = gizmo_origin;
                    let cursor_on_plane = if let Some(toi) = query::details::ray_toi_with_halfspace(
                        &plane_origin.into(),
                        &plane_normal.into(),
                        &query::Ray::new(ray_start.into(), ray_dir.into()),
                    ) {
                        ray_start + ray_dir * toi
                    } else {
                        return;
                    };

                    let cursor_vector: Vec3 = cursor_on_plane - plane_origin;
                    let cursor_projected_onto_handle = match &gizmo.drag_start {
                        Some(drag_start) => *drag_start,
                        None => {
                            let handle_vector = axis;
                            let cursor_projected_onto_handle = cursor_vector
                                .dot(handle_vector.normalize())
                                * handle_vector.normalize();
                            gizmo.drag_start = Some(cursor_projected_onto_handle + plane_origin);
                            return;
                        }
                    };
                    let selected_handle_vec = cursor_projected_onto_handle - plane_origin;
                    let new_handle_vec = cursor_vector.dot(selected_handle_vec.normalize())
                        * selected_handle_vec.normalize();
                    let translation = new_handle_vec - selected_handle_vec;
                    transform_queries
                        .p0()
                        .iter_mut()
                        .filter(|(s, _t, _i, _mprops)| s.selected())
                        .for_each(|(_s, mut t, i, _mprops)| {
                            let i = i.transform.compute_transform();
                            *t = Transform {
                                translation: i.translation + translation,
                                rotation: i.rotation,
                                scale: i.scale,
                            }
                        });
                }
                TransformGizmoInteraction::RotateAxis { original: _, axis } => {
                    let cursor_on_plane = if let Some(toi) = query::details::ray_toi_with_halfspace(
                        &gizmo_origin.into(),
                        &axis.normalize().into(),
                        &query::Ray::new(ray_start.into(), ray_dir.into()),
                    ) {
                        ray_start + ray_dir * toi
                    } else {
                        return;
                    };

                    let cursor_vector = (cursor_on_plane - gizmo_origin).normalize();
                    let drag_start = match &gizmo.drag_start {
                        Some(drag_start) => *drag_start,
                        None => {
                            gizmo.drag_start = Some(cursor_vector);
                            return; // We just started dragging, no transformation is needed yet, exit early.
                        }
                    };

                    let dot = drag_start.dot(cursor_vector);
                    let det = axis.dot(drag_start.cross(cursor_vector));
                    let angle = det.atan2(dot);
                    let rotation = Quat::from_axis_angle(axis, angle);
                    transform_queries
                        .p0()
                        .iter_mut()
                        .filter(|(s, _t, _i, _mprops)| s.selected())
                        .for_each(|(_s, mut t, i, mprops)| {
                            let i = i.transform.compute_transform();
                            if let Some(mprops) = mprops {
                                let world_com =
                                    i.translation + i.rotation * mprops.0.local_center_of_mass;
                                *t = Transform {
                                    translation: world_com
                                        + rotation * (-world_com + i.translation),
                                    rotation: rotation * i.rotation,
                                    scale: i.scale,
                                }
                            } else {
                                *t = Transform {
                                    translation: i.translation,
                                    rotation: rotation * i.rotation,
                                    scale: i.scale,
                                }
                            }
                        });
                }
                TransformGizmoInteraction::ScaleAxis {
                    original: _,
                    axis: _,
                } => (),
            }
        }
    }
}

fn hover_gizmo(
    scene_mouse: Res<SceneMouse>,
    mut gizmo_query: Query<(&mut TransformGizmo, &mut Interaction)>,
    hover_query: Query<(&Parent, &TransformGizmoInteraction)>,
    #[cfg(feature = "dim2")] mut gizmo_materials: Query<(
        &GizmoStateMaterials,
        &mut Handle<ColorMaterial>,
    )>,
    #[cfg(feature = "dim3")] mut gizmo_materials: Query<(
        &GizmoStateMaterials,
        &mut Handle<GizmoMaterial>,
    )>,
) {
    if let Ok((gizmo, _)) = gizmo_query.get_single() {
        if gizmo.initial_transform.is_some() {
            return;
        }
    }

    for (mats, mut out_mat) in gizmo_materials.iter_mut() {
        *out_mat = mats.idle.clone();
    }

    // NOTE: we only reach this point if we didn’t return earlier
    //       because the gizmo is still being hovered.
    for (_, mut interaction) in gizmo_query.iter_mut() {
        if *interaction == Interaction::Hovered {
            // dbg!("Reset hover");
            *interaction = Interaction::None
        }
    }

    if let Some(SelectableSceneObject::SelectionShape(entity)) = scene_mouse.hovered {
        if let Ok((mats, mut out_mat)) = gizmo_materials.get_mut(entity) {
            *out_mat = mats.hovered.clone();
        }

        if let Ok((parent, gizmo_interaction)) = hover_query.get(entity) {
            let (mut gizmo, mut interaction) = gizmo_query.get_mut(parent.get()).unwrap();
            // if *interaction == Interaction::None {
            // dbg!("Set hover");
            *interaction = Interaction::Hovered;
            gizmo.current_interaction = Some(*gizmo_interaction);
            // }

            return;
        }
    }
}

/// Tracks when one of the gizmo handles has been clicked on.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn grab_gizmo(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut gizmo_events: EventWriter<TransformGizmoEvent>,
    mut gizmo_query: Query<(&mut TransformGizmo, &mut Interaction, &GlobalTransform)>,
    selected_items_query: Query<(&Selection, &GlobalTransform, Entity)>,
    initial_transform_query: Query<Entity, With<InitialTransform>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (mut gizmo, mut interaction, _transform) in gizmo_query.iter_mut() {
            if *interaction == Interaction::Hovered {
                *interaction = Interaction::Clicked;
                // Dragging has started, store the initial position of all selected meshes
                for (selection, transform, entity) in selected_items_query.iter() {
                    if selection.selected() {
                        commands.entity(entity).insert(InitialTransform {
                            transform: *transform,
                        });
                    }
                }
            } else {
                *gizmo = TransformGizmo::default();
                for entity in initial_transform_query.iter() {
                    commands.entity(entity).remove::<InitialTransform>();
                }
            }
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        for (mut gizmo, mut interaction, transform) in gizmo_query.iter_mut() {
            *interaction = Interaction::None;
            if let (Some(from), Some(interaction)) =
                (gizmo.initial_transform, gizmo.current_interaction())
            {
                let event = TransformGizmoEvent {
                    from,
                    to: *transform,
                    interaction,
                };
                //info!("{:?}", event);
                gizmo_events.send(event);
                *gizmo = TransformGizmo::default();
            }
        }
    }
}

/// Places the gizmo in space relative to the selected entity(s).
#[allow(clippy::type_complexity)]
fn place_gizmo(
    plugin_settings: Res<GizmoSettings>,
    mut queries: ParamSet<(
        Query<
            (&Selection, &GlobalTransform, Option<&ReadMassProperties>),
            With<GizmoTransformable>,
        >,
        Query<(&mut GlobalTransform, &mut Transform, &mut Visibility), With<TransformGizmo>>,
    )>,
    #[cfg(feature = "dim2")] camera: Query<&OrbitCamera>,
) {
    let selected: Vec<_> = queries
        .p0()
        .iter()
        .filter(|(s, _t, _mprops)| s.selected())
        .map(|(_s, t, mprops)| {
            let t = t.compute_transform();
            mprops
                .map(|mprops| {
                    #[cfg(feature = "dim2")]
                    {
                        t.transform_point(Vec3::new(
                            mprops.0.local_center_of_mass.x,
                            mprops.0.local_center_of_mass.y,
                            0.0,
                        ))
                    }

                    #[cfg(feature = "dim3")]
                    {
                        t.transform_point(mprops.0.local_center_of_mass)
                    }
                })
                .unwrap_or(t.translation)
        })
        .collect();
    let n_selected = selected.len();
    let transform_sum = selected.iter().fold(Vec3::ZERO, |acc, t| acc + *t);
    // NOTE: mut is needed for dim2
    let mut centroid = transform_sum / n_selected as f32;
    // Set the gizmo's position and visibility
    if let Ok((mut g_transform, mut transform, mut visible)) = queries.p1().get_single_mut() {
        #[cfg(feature = "dim2")]
        {
            centroid.z = 1e-5; // Keep on top.

            let camera = camera.get_single().unwrap();
            transform.scale.x = 120.0 / camera.zoom;
            transform.scale.y = 120.0 / camera.zoom;
        }

        transform.translation = centroid;
        transform.rotation = plugin_settings.alignment_rotation;
        *g_transform = GlobalTransform::from(*transform);
        visible.is_visible = n_selected > 0;
    } else {
        error!("Number of gizmos is != 1");
    }
}

/// Updates the gizmo axes rotation based on the gizmo settings
fn update_gizmo_alignment(
    plugin_settings: Res<GizmoSettings>,
    mut query: Query<&mut TransformGizmoInteraction>,
) {
    let rotation = plugin_settings.alignment_rotation;
    for mut interaction in query.iter_mut() {
        if let Some(rotated_interaction) = match *interaction {
            TransformGizmoInteraction::TranslateAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::TranslateAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
            TransformGizmoInteraction::RotateAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::RotateAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
            TransformGizmoInteraction::ScaleAxis { original, axis: _ } => {
                Some(TransformGizmoInteraction::ScaleAxis {
                    original,
                    axis: rotation.mul_vec3(original),
                })
            }
        } {
            *interaction = rotated_interaction;
        }
    }
}

fn sync_gizmo_camera(
    main_cam: Query<
        (
            &Camera,
            &GlobalTransform,
            ChangeTrackers<GlobalTransform>,
            ChangeTrackers<Camera>,
        ),
        (With<MainCamera>, Without<GizmoCamera>),
    >,
    mut gizmo_cam: Query<(&mut Camera, &mut GlobalTransform), With<GizmoCamera>>,
) {
    let (main_cam, main_cam_pos, mcpos_change, mc_change) = main_cam.single();
    let (mut gizmo_cam, mut gizmo_cam_pos) = gizmo_cam.single_mut();
    if mcpos_change.is_changed() {
        *gizmo_cam_pos = *main_cam_pos;
    }
    if mc_change.is_changed() {
        *gizmo_cam = main_cam.clone();
        gizmo_cam.priority = GIZMO_LAYER as isize;
    }
}
