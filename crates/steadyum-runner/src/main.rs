#[cfg(feature = "dim2")]
extern crate rapier2d as rapier;
#[cfg(feature = "dim3")]
extern crate rapier3d as rapier;

mod cli;
mod commands;
mod connected_components;
mod neighbors;
mod region_assignment;
mod runner;
mod watch;

use crate::cli::CliArgs;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.init();

    let args = CliArgs::parse();
    let bounds = args.simulation_bounds();
    let (command_snd, command_rcv) = flume::unbounded();

    std::thread::spawn(move || {
        commands::start_command_loop(bounds, command_snd).unwrap();
    });
    runner::run_simulation(args, command_rcv)
}
