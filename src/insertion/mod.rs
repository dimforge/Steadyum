use crate::operation::Operation;
use crate::ui::SelectedTool;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_prototype_lyon::draw::Stroke;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_rapier::dynamics::RigidBody;
use bevy_rapier::rapier::math::DIM;
use bevy_rapier::{geometry::Collider, math::Vect};

use crate::render::RenderSystems;
use crate::utils::{ColliderBundle, RigidBodyBundle};
#[cfg(feature = "dim3")]
use bevy_polyline::prelude::*;
use na::DMatrix;
use noise::{NoiseFn, Perlin};

pub(self) const ACTIVE_EPS: f32 = 1.0e-1;

mod mouse;

#[derive(Component)]
pub struct InsertionPreview;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InsertionStep {
    Basis,
    Height,
    Orientation,
}

#[derive(Default, Clone, Resource)]
pub struct InsertionState {
    pub step: Option<InsertionStep>,
    pub preview_shape: Option<Collider>,
    pub basis: [Vect; DIM], // X, Y, Z
    pub start_point: Vect,
    pub end_point: Vect,
    #[cfg(feature = "dim3")]
    pub height: f32,
    pub on_empty_ground: bool,
    pub tool: SelectedTool,
    pub intersects_environment: bool,
    pub unlocked_scaling: bool,
}

impl InsertionState {
    pub fn normal(&self) -> Vect {
        self.basis[1]
    }

    pub fn transform(&self) -> Transform {
        #[cfg(feature = "dim2")]
        let mut scale = Vec2::new(
            (self.end_point - self.start_point).dot(self.basis[0]),
            (self.end_point - self.start_point).dot(self.basis[1]),
        );
        #[cfg(feature = "dim3")]
        let mut scale = Vec3::new(
            (self.end_point - self.start_point).dot(self.basis[0]),
            self.height,
            (self.end_point - self.start_point).dot(self.basis[2]),
        );

        if !self.unlocked_scaling {
            match self.tool {
                #[cfg(feature = "dim2")]
                SelectedTool::AddBall => {
                    let uniform_scale = scale.length() / std::f32::consts::SQRT_2;
                    scale.x = uniform_scale * scale.x.signum();
                    scale.y = uniform_scale * scale.y.signum();
                }
                #[cfg(feature = "dim3")]
                SelectedTool::AddBall => {
                    let uniform_scale = scale.xz().length() / std::f32::consts::SQRT_2;
                    scale.x = uniform_scale * scale.x.signum();
                    scale.z = uniform_scale * scale.z.signum();

                    // We want to keep the preview flat when we are still drawing the basis.
                    if self.step != Some(InsertionStep::Basis) {
                        scale.y = uniform_scale * scale.y.signum();
                    }
                }
                #[cfg(feature = "dim3")]
                SelectedTool::AddCylinder | SelectedTool::AddCapsule | SelectedTool::AddCone => {
                    let rad_scale = scale.xz().length() / std::f32::consts::SQRT_2;
                    scale.x = rad_scale * scale.x.signum();
                    scale.z = rad_scale * scale.z.signum();
                }
                _ => {}
            }
        }

        #[cfg(feature = "dim2")]
        {
            let translation =
                self.start_point + (self.basis[0] * scale.x + self.basis[1] * scale.y) / 2.0;
            Transform {
                translation: Vec3::new(translation.x, translation.y, 0.0),
                scale: Vec3::new(scale.x.abs(), scale.y.abs(), 1.0),
                rotation: Quat::IDENTITY, // TODO
            }
        }

        #[cfg(feature = "dim3")]
        {
            let translation = self.start_point
                + (self.basis[0] * scale.x + self.basis[1] * scale.y + self.basis[2] * scale.z)
                    / 2.0;
            Transform {
                translation,
                scale: scale.abs(),
                rotation: Quat::from_mat3(&Mat3::from_cols(
                    self.basis[0],
                    self.basis[1],
                    self.basis[2],
                )),
            }
        }
    }

    pub fn operation(&self) -> Operation {
        let rigid_body = if self.on_empty_ground {
            RigidBody::Fixed
        } else {
            RigidBody::Dynamic
        };

        let mut transform = self.transform();
        let mut collider = self.preview_shape.clone().unwrap();

        // For the capsule, we need to make a special-case so we
        // actually get a capsule if the scale is only non-uniform along
        // the Y axis.
        if self.tool == SelectedTool::AddCapsule {
            if (cfg!(feature = "dim2") || transform.scale.x == transform.scale.z)
                && transform.scale.y > transform.scale.x
            {
                collider = Collider::capsule_y(
                    (transform.scale.y - transform.scale.x) / 2.0,
                    transform.scale.x / 2.0,
                );
                transform.scale = Vec3::ONE;
            } else {
                let diameter = transform.scale.x.max(transform.scale.z);
                let height = (transform.scale.y - diameter).max(1.0e-3);
                collider = Collider::capsule_y(height / 2.0, diameter / 2.0);
                transform.scale.x /= diameter;
                transform.scale.y /= height + diameter;

                if cfg!(feature = "dim3") {
                    transform.scale.z /= diameter;
                }
            }
        } else if self.tool == SelectedTool::AddHeightfield {
            let perlin = Perlin::default();

            #[cfg(feature = "dim2")]
            {
                let num_rows = 100;
                let heights = (0..num_rows)
                    .map(|i| perlin.get([i as f64 / 100.0, 0.0]) as f32)
                    .collect();
                collider = Collider::heightfield(heights, Vec2::ONE);
            }

            #[cfg(feature = "dim3")]
            {
                let (num_rows, num_cols) = (100, 100);
                let heights = DMatrix::from_fn(num_rows, num_cols, |i, j| {
                    perlin.get([i as f64 / 100.0, j as f64 / 100.0]) as f32
                });
                collider = Collider::heightfield(
                    heights.data.as_vec().clone(),
                    num_rows,
                    num_cols,
                    Vec3::ONE,
                );
            }
        }

        Operation::AddCollider(
            ColliderBundle::new(collider),
            RigidBodyBundle {
                rigid_body,
                ..Default::default()
            },
            transform,
        )
    }

    pub fn set_tool(&mut self, tool: SelectedTool) {
        if tool != self.tool {
            self.tool = tool;

            match self.tool {
                #[cfg(feature = "dim2")]
                SelectedTool::AddCuboid | SelectedTool::AddHeightfield => {
                    self.preview_shape = Some(Collider::cuboid(0.5, 0.5))
                }
                #[cfg(feature = "dim3")]
                SelectedTool::AddCuboid | SelectedTool::AddHeightfield => {
                    self.preview_shape = Some(Collider::cuboid(0.5, 0.5, 0.5))
                }
                SelectedTool::AddBall => self.preview_shape = Some(Collider::ball(0.5)),
                SelectedTool::AddCapsule => {
                    self.preview_shape = Some(Collider::capsule_y(0.25, 0.25))
                }
                #[cfg(feature = "dim3")]
                SelectedTool::AddCylinder => {
                    self.preview_shape = Some(Collider::cylinder(0.5, 0.5))
                }
                #[cfg(feature = "dim3")]
                SelectedTool::AddCone => self.preview_shape = Some(Collider::cone(0.5, 0.5)),
                _ => self.preview_shape = None,
            }
        }
    }
}

pub struct InsertionPlugin;

impl Plugin for InsertionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InsertionState::default())
            .add_systems(Startup, spawn_preview_entity)
            .add_systems(
                Update,
                mouse::handle_insertion_click.in_set(RenderSystems::BeforeCommands),
            )
            .add_systems(
                Update,
                mouse::update_preview_scale.in_set(RenderSystems::BeforeCommands),
            );
    }
}

#[cfg(feature = "dim3")]
fn spawn_preview_entity(
    mut commands: Commands,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    let polyline = PolylineBundle {
        polyline: polylines.add(crate::styling::cuboid_polyline()),
        material: polyline_materials.add(PolylineMaterial {
            width: 20.0,
            perspective: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    commands
        .spawn(polyline)
        .insert(InsertionPreview)
        .insert(Visibility::Hidden);
}

#[cfg(feature = "dim2")]
fn spawn_preview_entity(mut commands: Commands) {
    commands
        .spawn(preview_shape_bundle(Vect::ONE, Color::WHITE))
        .insert(InsertionPreview)
        .insert(Visibility::Hidden);
}

#[cfg(feature = "dim2")]
pub fn preview_shape_bundle(
    scale: Vect,
    color: Color,
) -> (
    bevy_prototype_lyon::entity::ShapeBundle,
    bevy_prototype_lyon::prelude::Stroke,
) {
    use bevy_prototype_lyon::prelude::{GeometryBuilder, ShapeBundle, Stroke};

    let polyline = crate::styling::cuboid_polyline();
    let polygon = bevy_prototype_lyon::shapes::Polygon {
        points: polyline.vertices.iter().map(|pt| pt.xy() * scale).collect(),
        closed: true,
    };

    (
        ShapeBundle {
            path: GeometryBuilder::build_as(&polygon),
            ..default()
        },
        Stroke::new(color, 0.01),
    )
}
