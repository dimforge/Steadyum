use steadyum_api_types::simulation::SimulationBounds;

#[derive(clap::Parser, Debug, Copy, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value_t = -10_000, allow_negative_numbers = true)]
    pub xmin: i64,
    #[arg(long, default_value_t = 10_000, allow_negative_numbers = true)]
    pub xmax: i64,
    #[arg(long, default_value_t = -10_000, allow_negative_numbers = true)]
    pub ymin: i64,
    #[arg(long, default_value_t = 10_000, allow_negative_numbers = true)]
    pub ymax: i64,
    #[arg(long, default_value_t = -10_000, allow_negative_numbers = true)]
    #[cfg(feature = "dim3")]
    pub zmin: i64,
    #[arg(long, default_value_t = 10_000, allow_negative_numbers = true)]
    #[cfg(feature = "dim3")]
    pub zmax: i64,
    #[arg(long, default_value_t = 0)]
    pub time_origin: u64,
    #[arg(long, default_value_t = 10000, allow_negative_numbers = true)]
    pub port: u32,
}

impl CliArgs {
    #[cfg(feature = "dim2")]
    pub fn simulation_bounds(&self) -> SimulationBounds {
        SimulationBounds {
            mins: [self.xmin, self.ymin],
            maxs: [self.xmax, self.ymax],
        }
    }

    #[cfg(feature = "dim3")]
    pub fn simulation_bounds(&self) -> SimulationBounds {
        SimulationBounds {
            mins: [self.xmin, self.ymin, self.zmin],
            maxs: [self.xmax, self.ymax, self.zmax],
        }
    }
}
