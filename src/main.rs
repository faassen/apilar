extern crate num;
#[macro_use]
extern crate num_derive;
extern crate serde_big_array;

pub mod assembler;
pub mod client_command;
pub mod computer;
pub mod config;
pub mod direction;
pub mod info;
pub mod instruction;
pub mod island;
pub mod memory;
pub mod processor;
pub mod render;
pub mod run;
pub mod serve;
pub mod starter;
pub mod ticks;
pub mod world;

#[cfg(test)]
pub mod testutil;

use crate::assembler::{text_to_words, Assembler};
use crate::island::Island;
use crate::run::{load_command, run_command};
use crate::starter::PROGRAM_TEXT;
use crate::ticks::Ticks;
use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::fs::File;
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
    Load(Box<Load>),
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

    #[clap(long, default_value_t = Ticks(10000), value_parser = Ticks::parse)]
    mutation_frequency: Ticks,

    #[clap(long, default_value_t = 1, value_parser)]
    memory_overwrite_mutation_amount: u64,

    #[clap(long, default_value_t = 1, value_parser)]
    memory_insert_mutation_amount: u64,

    #[clap(long, default_value_t = 0, value_parser)]
    memory_delete_mutation_amount: u64,

    #[clap(long, default_value_t = 1, value_parser)]
    processor_stack_mutation_amount: u64,

    #[clap(long, default_value_t = 20000, value_parser)]
    death_rate: u32,

    #[clap(long, default_value_t = 2usize.pow(13), value_parser)]
    death_memory_size: usize,

    #[clap(long, default_value_t = 128, value_parser)]
    max_eat_amount: u64,

    #[clap(long, default_value_t = 16, value_parser)]
    max_grow_amount: u64,

    #[clap(long, default_value_t = 16, value_parser)]
    max_shrink_amount: u64,

    #[clap(long, default_value_t = false, value_parser)]
    autosave: bool,

    #[clap(long, default_value_t = 1000 * 60, value_parser)]
    save_frequency: u64,

    #[clap(long, default_value_t = 1000 / 8, value_parser)]
    redraw_frequency: u64,

    #[clap(long, default_value_t = false, value_parser)]
    text_ui: bool,

    #[clap(long, default_value_t = false, value_parser)]
    no_server: bool,
}

#[derive(Debug, Args)]
pub struct Load {
    #[clap(value_parser)]
    filename: String,

    #[clap(long, default_value_t = 10, value_parser)]
    instructions_per_update: usize,

    #[clap(long, default_value_t = Ticks(10000), value_parser = Ticks::parse)]
    mutation_frequency: Ticks,

    #[clap(long, default_value_t = 1, value_parser)]
    memory_overwrite_mutation_amount: u64,

    #[clap(long, default_value_t = 1, value_parser)]
    memory_insert_mutation_amount: u64,

    #[clap(long, default_value_t = 0, value_parser)]
    memory_delete_mutation_amount: u64,

    #[clap(long, default_value_t = 1, value_parser)]
    processor_stack_mutation_amount: u64,

    #[clap(long, default_value_t = 20000, value_parser)]
    death_rate: u32,

    #[clap(long, default_value_t = 2usize.pow(13), value_parser)]
    death_memory_size: usize,

    #[clap(long, default_value_t = 128, value_parser)]
    max_eat_amount: u64,

    #[clap(long, default_value_t = 16, value_parser)]
    max_grow_amount: u64,

    #[clap(long, default_value_t = 16, value_parser)]
    max_shrink_amount: u64,

    #[clap(long, default_value_t = false, value_parser)]
    autosave: bool,

    #[clap(long, default_value_t = 100000000, value_parser)]
    autosave_frequency: u64,

    #[clap(long, default_value_t = 100000, value_parser)]
    redraw_frequency: u64,

    #[clap(long, default_value_t = false, value_parser)]
    text_ui: bool,

    #[clap(long, default_value_t = false, value_parser)]
    no_server: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
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
            run_command(cli, words).await?;
        }
        Commands::Load(cli) => load_command(cli).await?,
        Commands::Disassemble { filename, x, y } => {
            let file = BufReader::new(File::open(filename)?);
            let island: Island = serde_cbor::from_reader(file)?;
            if *x >= island.width {
                println!("x out of range");
                return Ok(());
            }
            if *y >= island.height {
                println!("y out of range");
                return Ok(());
            }

            let location = island.get((*x, *y));
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
