use crate::assembler::Assembler;
use crate::computer::Computer;
use crate::simulation::{run as sim_run, Simulation};
use crate::world::World;
use crate::{Load, Run};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::Mutex;

pub async fn load(load: &Load) -> Result<(), Box<dyn Error + Sync + Send>> {
    let file = BufReader::new(File::open(load.filename.clone())?);

    let world: Arc<Mutex<World>> = Arc::new(Mutex::new(serde_cbor::from_reader(file)?));

    let simulation: Arc<Simulation> = Arc::new(Simulation::from(load));

    sim_run(simulation, world).await
}

pub async fn run(run: &Run, words: Vec<&str>) -> Result<(), Box<dyn Error + Sync + Send>> {
    let assembler = Assembler::new();

    let world = Arc::new(Mutex::new(World::new(
        run.width,
        run.height,
        run.world_resources,
    )));

    let mut computer = Computer::new(
        run.starting_memory_size,
        run.max_processors,
        run.starting_resources,
    );
    assembler.assemble_words(words, &mut computer.memory, 0);
    computer.add_processor(0);
    world
        .lock()
        .unwrap()
        .set((run.width / 2, run.height / 2), computer);

    let simulation = Arc::new(Simulation::from(run));

    sim_run(simulation, world).await
}
