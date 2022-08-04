use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::info::WorldInfo;
use crate::instruction::Metabolism;
use crate::render::{render_start, render_update};
use crate::serve::serve;
use crate::world::World;
use crate::{Load, Run};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time;

pub struct Frequencies {
    // how many ticks before a mutation; this could turn into a mutation
    // chance per tick
    pub mutation_frequency: u64,
    // this could be expressed as frames per second,
    // but better consistency is 'every x miliseconds'
    pub redraw_frequency: u64,
    // this could be expressed as "every 5 minutes"
    pub save_frequency: u64,
}

pub struct Simulation {
    pub instructions_per_update: usize,
    memory_mutation_amount: u64,
    processor_stack_mutation_amount: u64,
    pub death_rate: u32,
    pub metabolism: Metabolism,
    frequencies: Frequencies,
    dump: bool,
    text_ui: bool,
}

pub async fn run(
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
                world
                    .lock()
                    .unwrap()
                    .mutate_memory(&mut small_rng, simulation.memory_mutation_amount);
                world.lock().unwrap().mutate_memory_insert(&mut small_rng);
                // world.mutate_memory_delete(small_rng);
                world.lock().unwrap().mutate_processor_stack(
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

impl From<&Run> for Simulation {
    fn from(run: &Run) -> Self {
        Simulation {
            instructions_per_update: run.instructions_per_update,
            memory_mutation_amount: run.memory_mutation_amount,
            processor_stack_mutation_amount: run.processor_stack_mutation_amount,
            death_rate: run.death_rate,
            metabolism: Metabolism {
                max_eat_amount: run.max_eat_amount,
                max_grow_amount: run.max_grow_amount,
                max_shrink_amount: run.max_shrink_amount,
            },
            frequencies: Frequencies {
                mutation_frequency: run.mutation_frequency,
                redraw_frequency: run.redraw_frequency,
                save_frequency: run.save_frequency,
            },
            dump: run.dump,
            text_ui: run.text_ui,
        }
    }
}

impl From<&Load> for Simulation {
    fn from(load: &Load) -> Self {
        Simulation {
            instructions_per_update: load.instructions_per_update,
            memory_mutation_amount: load.memory_mutation_amount,
            processor_stack_mutation_amount: load.processor_stack_mutation_amount,
            death_rate: load.death_rate,
            metabolism: Metabolism {
                max_eat_amount: load.max_eat_amount,
                max_grow_amount: load.max_grow_amount,
                max_shrink_amount: load.max_shrink_amount,
            },
            frequencies: Frequencies {
                mutation_frequency: load.mutation_frequency,
                redraw_frequency: load.redraw_frequency,
                save_frequency: load.save_frequency,
            },
            dump: load.dump,
            text_ui: load.text_ui,
        }
    }
}
