extern crate num;
#[macro_use]
extern crate num_derive;
extern crate serde_big_array;

pub mod assembler;
pub mod client_command;
pub mod command;
pub mod computer;
pub mod config;
pub mod direction;
pub mod habitat;
pub mod info;
pub mod instruction;
pub mod island;
pub mod memory;
pub mod processor;
pub mod rectangle;
pub mod serve;
pub mod ticks;
pub mod topology;
pub mod want;
pub mod world;

#[cfg(test)]
pub mod testutil;

use crate::command::{load_command, run_command};
use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(Box<RunConfigArgs>),
    Load(Box<RunConfigArgs>),
}

#[derive(Debug, Args)]
pub struct RunConfigArgs {
    #[clap(value_parser)]
    filename: String,

    #[clap(long, default_value_t = false, value_parser)]
    autosave: bool,

    #[clap(long, default_value_t = 60 * 5, value_parser)]
    autosave_frequency: u64,

    #[clap(long, default_value_t = 1000 / 8, value_parser)]
    redraw_frequency: u64,

    #[clap(long, default_value_t = false, value_parser)]
    no_server: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run(cli) => run_command(cli)?,
        Commands::Load(cli) => load_command(cli)?,
    }

    Ok(())
}
