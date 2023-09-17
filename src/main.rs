#[cfg(feature = "dim2")]
extern crate bevy_rapier2d as bevy_rapier;
#[cfg(feature = "dim3")]
extern crate bevy_rapier3d as bevy_rapier;
extern crate nalgebra as na;

pub use bevy_rapier::parry;
pub use bevy_rapier::rapier;

use crate::camera::{OrbitCamera, OrbitCameraPlugin};
use crate::cli::CliArgs;
use crate::layers::GIZMO_LAYER;
use crate::ui::UiState;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::camera::Projection;
use bevy::render::view::RenderLayers;
use bevy::window::WindowId;
use bevy::winit::WinitWindows;
use bevy_egui::egui::Visuals;
use bevy_infinite_grid::GridShadowCamera;
use bevy_rapier::prelude::*;
use clap::Parser;
use winit::window::Icon;

mod camera;
mod floor;
mod insertion;
mod operation;
mod render;
mod selection;
mod styling;
mod ui;
mod utils;

mod storage;

// Workaround for bevy’s lack of visibility propagation from an entity to its children.
mod propagate_visibility;

mod builtin_scenes;
mod cli;
mod control;
mod drag;
mod layers;
mod projectile;

#[derive(Component)]
pub struct MainCamera;
#[derive(Component)]
pub struct GizmoCamera;

#[derive(Resource, Default)]
pub struct PhysicsProgress {
    pub simulated_time: Real,
    pub simulated_steps: usize,
    pub progress_limit: usize,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum SteadyumStages {
    PostPhysics,
    RenderStage,
}

fn main() {
    let args = CliArgs::parse();

    // let title = if cfg!(feature = "dim2") {
    //     "Steadyum 2D".to_string()
    // } else {
    //     "Steadyum 3D".to_string()
    // };

    let mut app = App::new();
    app
        /*.insert_resource(WindowDescriptor {
            title,
            ..Default::default()
        })*/
        .insert_resource(ClearColor(Color::rgb(0.55, 0.55, 0.55)))
        .insert_resource(args)
        .insert_resource(PhysicsProgress::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy::pbr::wireframe::WireframePlugin)
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(bevy_obj::ObjPlugin)
        .add_plugins(selection::SelectionPlugins)
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default().with_physics_scale(1.0))
        .add_plugin(RapierDebugRenderPlugin::default().disabled())
        .add_plugin(render::RapierRenderPlugin)
        .add_plugin(ui::RapierUiPlugin)
        .add_plugin(styling::StylingPlugin)
        .add_plugin(operation::RapierOperationsPlugin)
        // .add_plugin(bevy_prototype_lyon::prelude::ShapePlugin)
        .add_plugin(insertion::InsertionPlugin)
        .add_plugin(floor::FloorPlugin)
        .add_plugin(drag::DragPlugin)
        .add_plugin(projectile::ProjectilePlugin)
        .add_plugin(control::ControlPlugin)
        .add_stage_after(
            PhysicsStages::Writeback,
            SteadyumStages::PostPhysics,
            SystemStage::parallel(),
        )
        // .add_startup_system(set_window_icon)
        .add_startup_system(init_profiling_and_gravity)
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(propagate_visibility::visible_propagate_system);

    if args.distributed_physics {
        app.add_plugin(storage::StoragePlugin);
    }

    #[cfg(feature = "dim2")]
    {
        app.add_plugin(bevy_polyline::PolylinePlugin);
    }
    #[cfg(feature = "voxels")]
    {
        app.add_system_to_stage(SteadyumStages::PostPhysics, handle_fractures);
    }

    app.run();
}

// TODO: should be turn profiling off when the profiling window isn’t open?
fn init_profiling_and_gravity(
    mut config: ResMut<RapierConfiguration>,
    mut physics: ResMut<RapierContext>,
) {
    config.gravity.y = -9.81;
    physics.pipeline.counters.enable();
}

fn set_window_icon(windows: NonSendMut<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();

    // Here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/window_icon.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();
    primary.set_window_icon(Some(icon));
}

#[cfg(feature = "dim2")]
fn setup_graphics(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(10.0, 2.0, 10.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..Default::default()
        },
        ..Default::default()
    });

    let orbit = OrbitCamera {
        pan_sensitivity: 0.01,
        ..OrbitCamera::default()
    };

    let camera = Camera2dBundle::default();
    commands
        .spawn(camera)
        .insert(orbit)
        .insert(MainCamera)
        .insert(RenderLayers::layer(0));

    commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::None,
            },
            camera: Camera {
                priority: GIZMO_LAYER as isize,
                ..default()
            },
            ..default()
        })
        .insert(GizmoCamera)
        .insert(RenderLayers::layer(GIZMO_LAYER));
}

#[cfg(feature = "dim3")]
fn setup_graphics(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(10.0, 2.0, 10.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut orbit = OrbitCamera {
        pan_sensitivity: 4.0,
        rotate_sensitivity: 0.1,
        ..OrbitCamera::default()
    };
    look_at(&mut orbit, Vec3::new(5.0, 5.0, 5.0), Vec3::ZERO);
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_matrix(
                Mat4::look_at_rh(
                    Vec3::new(-3.0, 3.0, 1.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                )
                .inverse(),
            ),
            projection: Projection::Perspective(PerspectiveProjection {
                far: 10_000.0,
                ..PerspectiveProjection::default()
            }),
            ..Default::default()
        })
        .insert(orbit)
        .insert(MainCamera)
        .insert(GridShadowCamera)
        .insert(RenderLayers::layer(0));

    commands
        .spawn(Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::None,
                depth_load_op: bevy::core_pipeline::core_3d::Camera3dDepthLoadOp::Clear(0.),
            },
            camera: Camera {
                priority: GIZMO_LAYER as isize,
                ..default()
            },
            ..default()
        })
        .insert(GizmoCamera)
        .insert(RenderLayers::layer(GIZMO_LAYER));
}

#[cfg(feature = "dim2")]
pub fn look_at(camera: &mut OrbitCamera, at: Vec2, zoom: f32) {
    camera.center.x = at.x;
    camera.center.y = at.y;
    camera.zoom = zoom;
}

#[cfg(feature = "dim3")]
pub fn look_at(camera: &mut OrbitCamera, eye: Vec3, at: Vec3) {
    camera.center.x = at.x;
    camera.center.y = at.y;
    camera.center.z = at.z;

    let view_dir = eye - at;
    camera.distance = view_dir.length();

    if camera.distance > 0.0 {
        camera.y = (view_dir.y / camera.distance).acos();
        camera.x = (-view_dir.z).atan2(view_dir.x) - std::f32::consts::FRAC_PI_2;
    }
}

pub fn setup_physics(
    cli: Res<CliArgs>,
    mut config: ResMut<RapierConfiguration>,
    mut debug_render_context: ResMut<DebugRenderContext>,
) {
    config.physics_pipeline_active = false;
    config.query_pipeline_active = !cli.distributed_physics;
    debug_render_context.pipeline.style.rigid_body_axes_length = 0.5;
    debug_render_context.always_on_top = cfg!(feature = "dim2");
    debug_render_context.enabled = false;
}

// TODO: move this elsewhere
#[cfg(feature = "voxels")]
fn handle_fractures(
    mut commands: Commands,
    mut colors: ResMut<ColorGenerator>,
    mut events: EventReader<FractureEvent>,
) {
    for event in events.iter() {
        for fragment in &event.fragments {
            let color = colors.gen_color();
            // let outline_color = ColorGenerator::outline_color(color);

            commands
                .entity(*fragment)
                .insert(ColliderRender::from(color))
                // .insert(ColliderOutlineRender::new(outline_color, 0.02))
                .insert(Visibility::default())
                .insert(ComputedVisibility::default());
        }
    }
}
