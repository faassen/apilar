use crate::assembler::Assembler;
use crate::computer::Computer;
use crate::serve::serve;
use crate::simulation::Simulation;
use crate::world::World;
use crate::{Load, Run};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub async fn load(load: &Load) -> Result<(), Box<dyn Error>> {
    let file = BufReader::new(File::open(load.filename.clone())?);

    let mut world: World = serde_cbor::from_reader(file)?;

    let simulation = Simulation::from(load);

    run_world(&simulation, &mut world).await
}

pub async fn run(run: &Run, words: Vec<&str>) -> Result<(), Box<dyn Error>> {
    let assembler = Assembler::new();

    let mut world = World::new(run.width, run.height, run.world_resources);

    let mut computer = Computer::new(
        run.starting_memory_size,
        run.max_processors,
        run.starting_resources,
    );
    assembler.assemble_words(words, &mut computer.memory, 0);
    computer.add_processor(0);
    world.set((run.width / 2, run.height / 2), computer);

    let simulation = Simulation::from(run);
    run_world(&simulation, &mut world).await
}

pub async fn run_world(simulation: &Simulation, world: &mut World) -> Result<(), Box<dyn Error>> {
    let mut small_rng = SmallRng::from_entropy();

    let (world_info_tx, _) = broadcast::channel(32);
    let world_info_tx2 = world_info_tx.clone();
    let (client_command_tx, mut client_command_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        serve(world_info_tx, client_command_tx).await;
    });

    simulation
        .run(
            world,
            &mut small_rng,
            world_info_tx2,
            &mut client_command_rx,
        )
        .await
}
