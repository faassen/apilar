use crate::assembler::Assembler;
use crate::computer::Computer;
use crate::serve::serve;
use crate::simulation::{Frequencies, Simulation};
use crate::world::World;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::error::Error;

use tokio::sync::mpsc;

pub async fn run(
    width: usize,
    height: usize,
    starting_memory_size: usize,
    starting_resources: u64,
    max_processors: usize,
    world_resources: u64,
    instructions_per_update: usize,
    mutation_frequency: u64,
    redraw_frequency: u64,
    save_frequency: u64,
    memory_mutation_amount: u64,
    processor_stack_mutation_amount: u64,
    eat_amount: u64,
    death_rate: u32,
    dump: bool,
    words: Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    let assembler = Assembler::new();

    let mut computer = Computer::new(starting_memory_size, max_processors, starting_resources);
    assembler.assemble_words(words, &mut computer.memory, 0);
    computer.add_processor(0);

    let mut world = World::new(width, height, eat_amount, world_resources);
    world.set((width / 2, height / 2), computer);

    let mut small_rng = SmallRng::from_entropy();

    let (world_info_tx, world_info_rx) = mpsc::channel(32);
    let (client_command_tx, mut client_command_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        serve(world_info_rx, client_command_tx).await;
    });

    let frequencies = Frequencies {
        mutation_frequency,
        redraw_frequency,
        save_frequency,
    };

    let simulation = Simulation::new(
        instructions_per_update,
        memory_mutation_amount,
        processor_stack_mutation_amount,
        death_rate,
        frequencies,
        dump,
    );

    simulation
        .run(
            &mut world,
            &mut small_rng,
            world_info_tx,
            &mut client_command_rx,
        )
        .await
}
