use steadyum_api_types::simulation::SimulationBounds;
use uuid::Uuid;

#[derive(clap::Parser, Debug, Copy, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long)]
    pub uuid: u128,
    #[arg(long, default_value_t = 0)]
    pub time_origin: u64,
}

impl CliArgs {
    pub fn typed_uuid(&self) -> Uuid {
        Uuid::from_u128_le(self.uuid)
    }
}
