extern crate num;
#[macro_use]
extern crate num_derive;
extern crate serde_big_array;

pub mod assembler;
pub mod client_command;
pub mod computer;
pub mod direction;
pub mod info;
pub mod instruction;
pub mod memory;
pub mod processor;
pub mod render;
pub mod run;
pub mod serve;
pub mod simulation;
pub mod single;
pub mod starter;
pub mod world;

#[cfg(test)]
pub mod testutil;

use crate::assembler::{text_to_words, Assembler};
use crate::run::run;
use crate::starter::PROGRAM_TEXT;
use crate::world::World;
use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Read};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(Box<Run>),
    Disassemble {
        #[clap(value_parser)]
        filename: String,
        #[clap(value_parser)]
        x: usize,
        #[clap(value_parser)]
        y: usize,
    },
}

#[derive(Debug, Args)]
pub struct Run {
    #[clap(value_parser)]
    filename: Option<String>,

    #[clap(long, default_value_t = 70, value_parser)]
    width: usize,

    #[clap(long, default_value_t = 40, value_parser)]
    height: usize,

    #[clap(long, default_value_t = 300, value_parser)]
    starting_memory_size: usize,

    #[clap(long, default_value_t = 500, value_parser)]
    starting_resources: u64,

    #[clap(long, default_value_t = 10, value_parser)]
    max_processors: usize,

    #[clap(long, default_value_t = 400, value_parser)]
    world_resources: u64,

    #[clap(long, default_value_t = 10, value_parser)]
    instructions_per_update: usize,

    #[clap(long, default_value_t = 10000, value_parser)]
    mutation_frequency: u64,

    #[clap(long, default_value_t = 100000, value_parser)]
    redraw_frequency: u64,

    #[clap(long, default_value_t = 100000000, value_parser)]
    save_frequency: u64,

    #[clap(long, default_value_t = 1, value_parser)]
    memory_mutation_amount: u64,

    #[clap(long, default_value_t = 1, value_parser)]
    processor_stack_mutation_amount: u64,

    // XXX this is now superfluous
    #[clap(long, default_value_t = 100, value_parser)]
    eat_amount: u64,

    #[clap(long, default_value_t = 20000, value_parser)]
    death_rate: u32,

    #[clap(long, default_value_t = false, value_parser)]
    dump: bool,

    #[clap(long, default_value_t = false, value_parser)]
    text_ui: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run(cli) => {
            let contents = match cli.filename.clone() {
                Some(filename) => {
                    let mut file = File::open(filename)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    contents
                }
                None => PROGRAM_TEXT.to_string(),
            };
            let words = text_to_words(&contents);
            run(cli, words).await?;
        }
        Commands::Disassemble { filename, x, y } => {
            let file = BufReader::new(File::open(filename)?);
            let world: World = serde_cbor::from_reader(file)?;
            if *x >= world.width {
                println!("x out of range");
                return Ok(());
            }
            if *y >= world.height {
                println!("y out of range");
                return Ok(());
            }

            let location = world.get((*x, *y));
            match &location.computer {
                Some(computer) => {
                    let assembler = Assembler::new();
                    let text = assembler.line_disassemble(&computer.memory.values);
                    println!("{}", text);
                }
                None => {
                    println!("No computer at this location")
                }
            }
        }
    }

    Ok(())
}
