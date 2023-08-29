#[cfg(feature = "dim2")]
extern crate rapier2d as rapier;
#[cfg(feature = "dim3")]
extern crate rapier3d as rapier;

mod cli;
mod commands;
mod runner;
mod server;
mod watch;

use crate::cli::CliArgs;
use clap::Parser;

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.init();

    let args = CliArgs::parse();
    let bounds = args.simulation_bounds();
    let (command_snd, command_rcv) = flume::unbounded();

    let shared_state = runner::spawn_simulation(args, command_rcv, command_snd.clone());
    // watch::spawn_watch_loop(bounds, command_snd.clone());
    // dbg!("Spawning server");
    // server::spawn_server(args.port, shared_state);
    // dbg!("Server spawned");
    commands::start_command_loop(bounds, command_snd);
}
