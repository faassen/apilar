use crate::assembler::Assembler;
use crate::computer::Computer;
use crate::serve::serve;
use crate::simulation::{Frequencies, Simulation};
use crate::world::World;
use crate::Run;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::error::Error;
use tokio::sync::mpsc;

pub async fn run(run: &Run, words: Vec<&str>) -> Result<(), Box<dyn Error>> {
    let assembler = Assembler::new();

    let mut computer = Computer::new(
        run.starting_memory_size,
        run.max_processors,
        run.starting_resources,
    );
    assembler.assemble_words(words, &mut computer.memory, 0);
    computer.add_processor(0);

    let mut world = World::new(run.width, run.height, run.eat_amount, run.world_resources);
    world.set((run.width / 2, run.height / 2), computer);

    let mut small_rng = SmallRng::from_entropy();

    let (world_info_tx, world_info_rx) = mpsc::channel(32);
    let (client_command_tx, mut client_command_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        serve(world_info_rx, client_command_tx).await;
    });

    let frequencies = Frequencies {
        mutation_frequency: run.mutation_frequency,
        redraw_frequency: run.redraw_frequency,
        save_frequency: run.save_frequency,
    };

    let simulation = Simulation::new(
        run.instructions_per_update,
        run.memory_mutation_amount,
        run.processor_stack_mutation_amount,
        run.death_rate,
        frequencies,
        run.dump,
        run.text_ui,
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
