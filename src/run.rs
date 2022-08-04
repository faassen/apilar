use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::computer::Computer;
use crate::info::WorldInfo;
use crate::render::{render_start, render_update};
use crate::serve::serve;
use crate::simulation::Simulation;
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

pub async fn load_command(cli: &Load) -> Result<(), Box<dyn Error + Sync + Send>> {
    let file = BufReader::new(File::open(cli.filename.clone())?);

    let world: Arc<Mutex<World>> = Arc::new(Mutex::new(serde_cbor::from_reader(file)?));

    let simulation: Arc<Simulation> = Arc::new(Simulation::from(cli));

    run(simulation, world).await
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

    let simulation = Arc::new(Simulation::from(cli));

    run(simulation, world).await
}

async fn run(
    simulation: Arc<Simulation>,
    world: Arc<Mutex<World>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut small_rng = SmallRng::from_entropy();

    let (world_info_tx, _) = broadcast::channel(32);
    let world_info_tx2 = world_info_tx.clone();
    let (client_command_tx, mut client_command_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        serve(world_info_tx, client_command_tx).await;
    });

    if simulation.text_ui {
        render_start();
    }

    let render_world = Arc::clone(&world);

    tokio::spawn(async move {
        loop {
            let _ = world_info_tx2.send(WorldInfo::new(&*render_world.lock().unwrap()));
            time::sleep(Duration::new(0, 1_000_000_000u32 / 8)).await;
        }
    });

    if simulation.dump {
        let mut save_nr = 0;
        let save_world = Arc::clone(&world);
        tokio::spawn(async move {
            loop {
                let result = save_file(&*save_world.lock().unwrap(), save_nr);
                if result.is_err() {
                    println!("Could not write save file");
                    break;
                }
                save_nr += 1;
                time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    if simulation.text_ui {
        let text_ui_world = Arc::clone(&world);
        tokio::spawn(async move {
            loop {
                render_update();
                println!("{}", text_ui_world.lock().unwrap());
                time::sleep(Duration::new(0, 1_000_000_000u32 / 8)).await;
            }
        });
    }

    let (main_loop_control_tx, mut main_loop_control_rx) = mpsc::channel::<bool>(32);
    let client_command_world = Arc::clone(&world);
    tokio::spawn(async move {
        while let Some(cmd) = client_command_rx.recv().await {
            match cmd {
                ClientCommand::Stop => {
                    main_loop_control_tx.send(false).await.unwrap();
                }
                ClientCommand::Start => {
                    main_loop_control_tx.send(true).await.unwrap();
                }
                ClientCommand::Disassemble { x, y, respond } => {
                    respond
                        .send(disassemble(&*client_command_world.lock().unwrap(), x, y))
                        .unwrap();
                }
            }
        }
    });

    tokio::task::spawn_blocking(move || {
        let mut tick: u64 = 0;

        loop {
            let mutate = tick % simulation.frequencies.mutation_frequency == 0;
            let receive_command = tick % simulation.frequencies.redraw_frequency == 0;

            world.lock().unwrap().update(&mut small_rng, &simulation);
            if mutate {
                let mut world = world.lock().unwrap();
                world.mutate_memory(&mut small_rng, simulation.memory_mutation_amount);
                world.mutate_memory_insert(&mut small_rng);
                // world.mutate_memory_delete(small_rng);
                world.mutate_processor_stack(
                    &mut small_rng,
                    simulation.processor_stack_mutation_amount,
                )
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
            tick = tick.wrapping_add(1);
        }
    })
    .await?
}

fn save_file(world: &World, save_nr: u64) -> Result<(), serde_cbor::Error> {
    let file = BufWriter::new(File::create(format!("apilar-dump{:06}.cbor", save_nr))?);
    serde_cbor::to_writer(file, world)
}

fn disassemble(world: &World, x: usize, y: usize) -> Result<String, String> {
    let assembler = Assembler::new();
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
