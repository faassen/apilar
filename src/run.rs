use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::computer::Computer;
use crate::config::Config;
use crate::habitat::{Habitat, HabitatConfig};
use crate::info::HabitatInfo;
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

    let habitat: Arc<Mutex<Habitat>> = Arc::new(Mutex::new(serde_cbor::from_reader(file)?));

    let config: Config = Config::from(cli);

    let assembler = Assembler::new();

    run(config, habitat, assembler).await
}

pub async fn run_command(cli: &Run, words: Vec<&str>) -> Result<(), Box<dyn Error + Sync + Send>> {
    let assembler = Assembler::new();

    let habitat = Arc::new(Mutex::new(Habitat::new(
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
    habitat
        .lock()
        .unwrap()
        .set((cli.width / 2, cli.height / 2), computer);

    let config = Config::from(cli);

    run(config, habitat, assembler).await
}

async fn run(
    config: Config,
    habitat: Arc<Mutex<Habitat>>,
    assembler: Assembler,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut rng = SmallRng::from_entropy();

    let (habitat_info_tx, _) = broadcast::channel(32);
    let (client_command_tx, client_command_rx) = mpsc::channel(32);

    if config.server {
        tokio::spawn(serve_task(habitat_info_tx.clone(), client_command_tx));
    }

    tokio::spawn(render_habitat_task(
        Arc::clone(&habitat),
        habitat_info_tx.clone(),
        config.redraw_frequency,
    ));

    let (main_loop_control_tx, main_loop_control_rx) = mpsc::channel::<bool>(32);

    tokio::spawn(client_command_task(
        Arc::clone(&habitat),
        assembler,
        client_command_rx,
        main_loop_control_tx,
    ));

    if config.autosave.enabled {
        tokio::spawn(save_habitat_task(
            Arc::clone(&habitat),
            config.autosave.frequency,
        ));
    }

    if config.text_ui {
        render_start();
        tokio::spawn(text_ui_task(Arc::clone(&habitat), config.redraw_frequency));
    }

    tokio::task::spawn_blocking(move || {
        simulation_task(
            config.habitat_config,
            Arc::clone(&habitat),
            &mut rng,
            main_loop_control_rx,
        )
    })
    .await?;
    Ok(())
}

async fn render_habitat_task(
    habitat: Arc<Mutex<Habitat>>,
    tx: broadcast::Sender<HabitatInfo>,
    duration: Duration,
) {
    loop {
        let _ = tx.send(HabitatInfo::new(&*habitat.lock().unwrap()));
        time::sleep(duration).await;
    }
}

async fn save_habitat_task(habitat: Arc<Mutex<Habitat>>, duration: Duration) {
    let mut save_nr = 0;
    loop {
        let result = save_habitat(&*habitat.lock().unwrap(), save_nr);
        if result.is_err() {
            println!("Could not write save file");
            break;
        }
        save_nr += 1;
        time::sleep(duration).await;
    }
}

async fn text_ui_task(habitat: Arc<Mutex<Habitat>>, duration: Duration) {
    loop {
        render_update();
        println!("{}", habitat.lock().unwrap());
        time::sleep(duration).await;
    }
}

async fn client_command_task(
    habitat: Arc<Mutex<Habitat>>,
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
                    .send(disassemble(&*habitat.lock().unwrap(), &assembler, x, y))
                    .unwrap();
            }
        }
    }
}

fn simulation_task(
    config: HabitatConfig,
    habitat: Arc<Mutex<Habitat>>,
    rng: &mut SmallRng,
    mut main_loop_control_rx: mpsc::Receiver<bool>,
) {
    let mut ticks = Ticks(0);

    loop {
        let mutate = ticks.is_at(config.mutation_frequency);
        let receive_command = ticks.is_at(COMMAND_PROCESS_FREQUENCY);

        habitat.lock().unwrap().update(rng, &config);
        if mutate {
            habitat.lock().unwrap().mutate(rng, &config.mutation);
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

fn save_habitat(habitat: &Habitat, save_nr: u64) -> Result<(), serde_cbor::Error> {
    let file = BufWriter::new(File::create(format!("apilar-dump{:06}.cbor", save_nr))?);
    serde_cbor::to_writer(file, habitat)
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
