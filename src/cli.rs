use crate::parry::bounding_volume::Aabb;
use bevy::prelude::*;
use clap::Parser;
use nalgebra::point;

#[derive(Parser, Debug, Copy, Clone, Resource)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value_t = -100_000.0)]
    xmin: f32,
    #[arg(long, default_value_t = 100_000.0)]
    xmax: f32,
    #[arg(long, default_value_t = -100_000.0)]
    ymin: f32,
    #[arg(long, default_value_t = 100_000.0)]
    ymax: f32,
    #[arg(long, default_value_t = -100_000.0)]
    zmin: f32,
    #[arg(long, default_value_t = 100_000.0)]
    zmax: f32,
    #[arg(long, default_value_t = false)]
    pub distributed_physics: bool,
    #[arg(long, default_value_t = false)]
    pub lower_graphics: bool,
}

impl CliArgs {
    #[cfg(feature = "dim2")]
    pub fn simulation_bounds(&self) -> Aabb {
        let mins = point![self.xmin, self.ymin];
        let maxs = point![self.xmax, self.ymax];
        Aabb::new(mins, maxs)
    }

    #[cfg(feature = "dim3")]
    pub fn simulation_bounds(&self) -> Aabb {
        let mins = point![self.xmin, self.ymin, self.zmin];
        let maxs = point![self.xmax, self.ymax, self.zmax];
        Aabb::new(mins, maxs)
    }

    pub fn awareness_bounds(&self) -> Aabb {
        let mut result = self.simulation_bounds();
        let sim_extents = result.extents();
        result.mins -= sim_extents;
        result.maxs += sim_extents;
        result
    }
}
