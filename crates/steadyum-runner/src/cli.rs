use steadyum_api_types::simulation::SimulationBounds;

#[derive(clap::Parser, Debug, Copy, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value_t = 0)]
    pub time_origin: u64,
}
