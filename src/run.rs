use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::computer::Computer;
use crate::config::{Config, RunConfig};
use crate::habitat::Habitat;
use crate::info::HabitatInfo;
use crate::render::{render_start, render_update};
use crate::serve::serve_task;
use crate::ticks::Ticks;
use crate::topology::Topology;
use crate::world::World;
use crate::{Load, Run, TopologyConfig};
use anyhow::Result;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time;

#[derive(Debug)]
pub struct Autosave {
    pub enabled: bool,
    pub frequency: Duration,
}

const COMMAND_PROCESS_FREQUENCY: Ticks = Ticks(10000);

pub async fn load_command(cli: &Load) -> Result<()> {
    let file = BufReader::new(File::open(cli.filename.clone())?);

    let run_config: RunConfig = RunConfig::from(cli);
    let world: World = serde_cbor::from_reader(file)?;

    let assembler = Assembler::new();

    run(run_config, Arc::new(Mutex::new(world)), assembler).await
}

pub async fn run_config(cli: &TopologyConfig) -> Result<()> {
    let file = BufReader::new(File::open(cli.filename.clone())?);
    let topology: Topology = serde_json::from_reader(file)?;
    let world = World::try_from(&topology)?;
    let run_config = RunConfig::from(cli);
    let assembler = Assembler::new();
    run(run_config, Arc::new(Mutex::new(world)), assembler).await
}

pub async fn run_command(cli: &Run, words: Vec<&str>) -> Result<()> {
    let assembler = Assembler::new();

    let config = Config::from(cli);

    let mut habitat = Habitat::new(cli.width, cli.height, cli.world_resources);

    let mut computer = Computer::new(cli.starting_memory_size, cli.starting_resources);
    assembler.assemble_words(words, &mut computer.memory, 0);
    computer.add_processor(0);
    habitat.set((cli.width / 2, cli.height / 2), computer);

    let world: Arc<Mutex<World>> = Arc::new(Mutex::new(World::from_habitat(
        habitat,
        config.habitat_config,
    )));

    run(config.run_config, world, assembler).await
}

async fn run(run_config: RunConfig, world: Arc<Mutex<World>>, assembler: Assembler) -> Result<()> {
    let mut rng = SmallRng::from_entropy();

    let (habitat_info_tx, _) = broadcast::channel(32);
    let (client_command_tx, client_command_rx) = mpsc::channel(32);

    if run_config.server {
        tokio::spawn(serve_task(habitat_info_tx.clone(), client_command_tx));
    }

    tokio::spawn(render_world_task(
        Arc::clone(&world),
        habitat_info_tx.clone(),
        run_config.redraw_frequency,
    ));

    let (main_loop_control_tx, main_loop_control_rx) = mpsc::channel::<bool>(32);

    tokio::spawn(client_command_task(
        Arc::clone(&world),
        assembler,
        client_command_rx,
        main_loop_control_tx,
    ));

    if run_config.autosave.enabled {
        tokio::spawn(save_world_task(
            Arc::clone(&world),
            run_config.autosave.frequency,
        ));
    }

    if run_config.text_ui {
        render_start();
        tokio::spawn(text_ui_task(
            Arc::clone(&world),
            run_config.redraw_frequency,
        ));
    }

    tokio::task::spawn_blocking(move || {
        simulation_task(Arc::clone(&world), &mut rng, main_loop_control_rx)
    })
    .await?;
    Ok(())
}

async fn render_world_task(
    world: Arc<Mutex<World>>,
    tx: broadcast::Sender<HabitatInfo>,
    duration: Duration,
) {
    loop {
        let _ = tx.send(HabitatInfo::new(&*world.lock().unwrap().habitat()));
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
        println!("{}", world.lock().unwrap().habitat());
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
                    .send(disassemble(
                        &*world.lock().unwrap().habitat(),
                        &assembler,
                        x,
                        y,
                    ))
                    .unwrap();
            }
        }
    }
}

fn simulation_task(
    world: Arc<Mutex<World>>,
    rng: &mut SmallRng,
    mut main_loop_control_rx: mpsc::Receiver<bool>,
) {
    let mut ticks = Ticks(0);

    loop {
        let receive_command = ticks.is_at(COMMAND_PROCESS_FREQUENCY);

        world.lock().unwrap().update(ticks, rng);

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

fn disassemble(
    habitat: &Habitat,
    assembler: &Assembler,
    x: usize,
    y: usize,
) -> Result<String, String> {
    if x >= habitat.width {
        return Err("x out of range".to_string());
    }
    if y >= habitat.height {
        return Err("y out of range".to_string());
    }

    let location = habitat.get((x, y));
    if let Some(computer) = &location.computer {
        Ok(assembler.line_disassemble(&computer.memory.values))
    } else {
        Err("no computer".to_string())
    }
}
