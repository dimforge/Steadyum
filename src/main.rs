use crate::camera::{OrbitCamera, OrbitCameraPlugin};
use crate::cli::CliArgs;
use crate::layers::GIZMO_LAYER;
use crate::physics::{PhysicsState, Real};
use crate::ui::UiState;
use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use clap::Parser;

mod camera;
mod floor;
mod insertion;
mod operation;
pub mod physics;
mod render;
mod selection;
mod styling;
mod ui;
mod utils;

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

fn main() {
    let args = CliArgs::parse();

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.55, 0.55, 0.55)))
        .insert_resource(args)
        .insert_resource(PhysicsProgress::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy::pbr::wireframe::WireframePlugin::default())
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(selection::SelectionPlugins)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(render::RapierRenderPlugin)
        .add_plugins(ui::RapierUiPlugin)
        .add_plugins(styling::StylingPlugin)
        .add_plugins(operation::RapierOperationsPlugin)
        .add_plugins(insertion::InsertionPlugin)
        .add_plugins(floor::FloorPlugin)
        .add_plugins(drag::DragPlugin)
        .add_plugins(projectile::ProjectilePlugin)
        .add_plugins(control::ControlPlugin)
        .add_plugins(OrbitCameraPlugin)
        .add_systems(Startup, init_profiling_and_gravity)
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics);

    app.run();
}

fn init_profiling_and_gravity(mut physics: ResMut<PhysicsState>) {
    physics.gravity.y = -9.81;
    physics.counters.enable();
}

#[cfg(feature = "dim2")]
fn setup_graphics(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(10.0, 2.0, 10.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..Default::default()
        },
    ));

    let orbit = OrbitCamera {
        pan_sensitivity: 0.01,
        ..OrbitCamera::default()
    };

    commands
        .spawn(Camera2d)
        .insert(orbit)
        .insert(MainCamera)
        .insert(RenderLayers::layer(0));

    commands
        .spawn((
            Camera2d,
            Camera {
                order: GIZMO_LAYER as isize,
                ..default()
            },
        ))
        .insert(GizmoCamera)
        .insert(RenderLayers::layer(GIZMO_LAYER));
}

#[cfg(feature = "dim3")]
fn setup_graphics(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(10.0, 2.0, 10.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..Default::default()
        },
        Name::new("Light"),
    ));

    let mut orbit = OrbitCamera {
        pan_sensitivity: 4.0,
        rotate_sensitivity: 0.1,
        ..OrbitCamera::default()
    };
    look_at(&mut orbit, Vec3::new(5.0, 5.0, 5.0), Vec3::ZERO);
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_matrix(
                Mat4::look_at_rh(
                    Vec3::new(-3.0, 3.0, 1.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                )
                .inverse(),
            ),
            Projection::Perspective(PerspectiveProjection {
                far: 100.0,
                ..PerspectiveProjection::default()
            }),
            Name::new("3D Camera"),
            orbit,
            MainCamera,
            RenderLayers::layer(0),
        ));

    commands
        .spawn((
            Camera3d {
                depth_load_op: bevy::camera::Camera3dDepthLoadOp::Clear(0.),
                ..default()
            },
            Camera {
                order: GIZMO_LAYER as isize,
                ..default()
            },
            Name::new("Gizmos Camera"),
            GizmoCamera,
            RenderLayers::layer(GIZMO_LAYER),
        ));
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

pub fn setup_physics(mut physics: ResMut<PhysicsState>) {
    physics.running = false;
}
