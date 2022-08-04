use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::computer::Computer;
use crate::configuration::Configuration;
use crate::info::IslandInfo;
use crate::island::Island;
use crate::render::{render_start, render_update};
use crate::serve::serve_task;
use crate::ticks::Ticks;
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

    let island: Arc<Mutex<Island>> = Arc::new(Mutex::new(serde_cbor::from_reader(file)?));

    let config: Configuration = Configuration::from(cli);

    let assembler = Assembler::new();

    run(config, island, assembler).await
}

pub async fn run_command(cli: &Run, words: Vec<&str>) -> Result<(), Box<dyn Error + Sync + Send>> {
    let assembler = Assembler::new();

    let island = Arc::new(Mutex::new(Island::new(
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
    island
        .lock()
        .unwrap()
        .set((cli.width / 2, cli.height / 2), computer);

    let config = Configuration::from(cli);

    run(config, island, assembler).await
}

async fn run(
    config: Configuration,
    island: Arc<Mutex<Island>>,
    assembler: Assembler,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut rng = SmallRng::from_entropy();

    let (island_info_tx, _) = broadcast::channel(32);
    let (client_command_tx, client_command_rx) = mpsc::channel(32);

    if config.server {
        tokio::spawn(serve_task(island_info_tx.clone(), client_command_tx));
    }

    tokio::spawn(render_island_task(
        Arc::clone(&island),
        island_info_tx.clone(),
        config.redraw_frequency,
    ));

    let (main_loop_control_tx, main_loop_control_rx) = mpsc::channel::<bool>(32);

    tokio::spawn(client_command_task(
        Arc::clone(&island),
        assembler,
        client_command_rx,
        main_loop_control_tx,
    ));

    if config.autosave.enabled {
        tokio::spawn(save_island_task(
            Arc::clone(&island),
            config.autosave.frequency,
        ));
    }

    if config.text_ui {
        render_start();
        tokio::spawn(text_ui_task(Arc::clone(&island), config.redraw_frequency));
    }

    tokio::task::spawn_blocking(move || {
        simulation_task(config, Arc::clone(&island), &mut rng, main_loop_control_rx)
    })
    .await?;
    Ok(())
}

async fn render_island_task(
    island: Arc<Mutex<Island>>,
    tx: broadcast::Sender<IslandInfo>,
    duration: Duration,
) {
    loop {
        let _ = tx.send(IslandInfo::new(&*island.lock().unwrap()));
        time::sleep(duration).await;
    }
}

async fn save_island_task(island: Arc<Mutex<Island>>, duration: Duration) {
    let mut save_nr = 0;
    loop {
        let result = save_island(&*island.lock().unwrap(), save_nr);
        if result.is_err() {
            println!("Could not write save file");
            break;
        }
        save_nr += 1;
        time::sleep(duration).await;
    }
}

async fn text_ui_task(island: Arc<Mutex<Island>>, duration: Duration) {
    loop {
        render_update();
        println!("{}", island.lock().unwrap());
        time::sleep(duration).await;
    }
}

async fn client_command_task(
    island: Arc<Mutex<Island>>,
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
                    .send(disassemble(&*island.lock().unwrap(), &assembler, x, y))
                    .unwrap();
            }
        }
    }
}

fn simulation_task(
    config: Configuration,
    island: Arc<Mutex<Island>>,
    rng: &mut SmallRng,
    mut main_loop_control_rx: mpsc::Receiver<bool>,
) {
    let mut ticks = Ticks(0);

    loop {
        let mutate = ticks.is_at(config.mutation_frequency);
        let receive_command = ticks.is_at(COMMAND_PROCESS_FREQUENCY);

        island.lock().unwrap().update(rng, &config);
        if mutate {
            island.lock().unwrap().mutate(rng, &config.mutation);
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

fn save_island(island: &Island, save_nr: u64) -> Result<(), serde_cbor::Error> {
    let file = BufWriter::new(File::create(format!("apilar-dump{:06}.cbor", save_nr))?);
    serde_cbor::to_writer(file, island)
}

fn disassemble(
    island: &Island,
    assembler: &Assembler,
    x: usize,
    y: usize,
) -> Result<String, String> {
    if x >= island.width {
        return Err("x out of range".to_string());
    }
    if y >= island.height {
        return Err("y out of range".to_string());
    }

    let location = island.get((x, y));
    if let Some(computer) = &location.computer {
        Ok(assembler.line_disassemble(&computer.memory.values))
    } else {
        Err("no computer".to_string())
    }
}
