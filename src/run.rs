use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::computer::Computer;
use crate::info::WorldInfo;
use crate::render::{render_start, render_update};
use crate::serve::serve_task;
use crate::simulation::Simulation;
use crate::ticks::Ticks;
use crate::world::World;
use crate::{Load, Run};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time;

pub struct Autosave {
    pub enabled: bool,
    pub frequency: Duration,
}

const COMMAND_PROCESS_FREQUENCY: Ticks = Ticks(10000);

pub async fn load_command(cli: &Load) -> Result<(), Box<dyn Error + Sync + Send>> {
    let file = BufReader::new(File::open(cli.filename.clone())?);

    let world: Arc<Mutex<World>> = Arc::new(Mutex::new(serde_cbor::from_reader(file)?));

    let simulation: Simulation = Simulation::from(cli);

    let assembler = Assembler::new();

    run(simulation, world, assembler).await
}

pub async fn run_command(cli: &Run, words: Vec<&str>) -> Result<(), Box<dyn Error + Sync + Send>> {
    let assembler = Assembler::new();

    let world = Arc::new(Mutex::new(World::new(
        cli.width,
        cli.height,
        cli.world_resources,
    )));

    let mut computer = Computer::new(
        cli.starting_memory_size,
        cli.max_processors,
        cli.starting_resources,
    );
    assembler.assemble_words(words, &mut computer.memory, 0);
    computer.add_processor(0);
    world
        .lock()
        .unwrap()
        .set((cli.width / 2, cli.height / 2), computer);

    let simulation = Simulation::from(cli);

    run(simulation, world, assembler).await
}

async fn run(
    simulation: Simulation,
    world: Arc<Mutex<World>>,
    assembler: Assembler,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut rng = SmallRng::from_entropy();

    let (world_info_tx, _) = broadcast::channel(32);
    let (client_command_tx, client_command_rx) = mpsc::channel(32);

    tokio::spawn(serve_task(world_info_tx.clone(), client_command_tx));

    tokio::spawn(render_world_task(
        Arc::clone(&world),
        world_info_tx.clone(),
        simulation.redraw_frequency,
    ));

    let (main_loop_control_tx, main_loop_control_rx) = mpsc::channel::<bool>(32);

    tokio::spawn(client_command_task(
        Arc::clone(&world),
        assembler,
        client_command_rx,
        main_loop_control_tx,
    ));

    if simulation.autosave.enabled {
        tokio::spawn(save_world_task(
            Arc::clone(&world),
            simulation.autosave.frequency,
        ));
    }

    if simulation.text_ui {
        render_start();
        tokio::spawn(text_ui_task(
            Arc::clone(&world),
            simulation.redraw_frequency,
        ));
    }

    tokio::task::spawn_blocking(move || {
        simulation_task(
            simulation,
            Arc::clone(&world),
            &mut rng,
            main_loop_control_rx,
        )
    })
    .await?;
    Ok(())
}

async fn render_world_task(
    world: Arc<Mutex<World>>,
    tx: broadcast::Sender<WorldInfo>,
    duration: Duration,
) {
    loop {
        let _ = tx.send(WorldInfo::new(&*world.lock().unwrap()));
        time::sleep(duration).await;
    }
}

async fn save_world_task(world: Arc<Mutex<World>>, duration: Duration) {
    let mut save_nr = 0;
    loop {
        let result = save_world(&*world.lock().unwrap(), save_nr);
        if result.is_err() {
            println!("Could not write save file");
            break;
        }
        save_nr += 1;
        time::sleep(duration).await;
    }
}

async fn text_ui_task(world: Arc<Mutex<World>>, duration: Duration) {
    loop {
        render_update();
        println!("{}", world.lock().unwrap());
        time::sleep(duration).await;
    }
}

async fn client_command_task(
    world: Arc<Mutex<World>>,
    assembler: Assembler,
    mut rx: mpsc::Receiver<ClientCommand>,
    tx: mpsc::Sender<bool>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            ClientCommand::Stop => {
                tx.send(false).await.unwrap();
            }
            ClientCommand::Start => {
                tx.send(true).await.unwrap();
            }
            ClientCommand::Disassemble { x, y, respond } => {
                respond
                    .send(disassemble(&*world.lock().unwrap(), &assembler, x, y))
                    .unwrap();
            }
        }
    }
}

fn simulation_task(
    simulation: Simulation,
    world: Arc<Mutex<World>>,
    rng: &mut SmallRng,
    mut main_loop_control_rx: mpsc::Receiver<bool>,
) {
    let mut ticks = Ticks(0);

    loop {
        let mutate = ticks.is_at(simulation.mutation_frequency);
        let receive_command = ticks.is_at(COMMAND_PROCESS_FREQUENCY);

        world.lock().unwrap().update(rng, &simulation);
        if mutate {
            world.lock().unwrap().mutate(rng, &simulation.mutation);
        }

        if receive_command {
            if let Ok(started) = main_loop_control_rx.try_recv() {
                if !started {
                    while let Some(started) = main_loop_control_rx.blocking_recv() {
                        if started {
                            break;
                        }
                    }
                }
            }
        }
        ticks = ticks.tick();
    }
}

fn save_world(world: &World, save_nr: u64) -> Result<(), serde_cbor::Error> {
    let file = BufWriter::new(File::create(format!("apilar-dump{:06}.cbor", save_nr))?);
    serde_cbor::to_writer(file, world)
}

fn disassemble(world: &World, assembler: &Assembler, x: usize, y: usize) -> Result<String, String> {
    if x >= world.width {
        return Err("x out of range".to_string());
    }
    if y >= world.height {
        return Err("y out of range".to_string());
    }

    let location = world.get((x, y));
    if let Some(computer) = &location.computer {
        Ok(assembler.line_disassemble(&computer.memory.values))
    } else {
        Err("no computer".to_string())
    }
}
