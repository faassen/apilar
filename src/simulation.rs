use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::info::WorldInfo;
use crate::instruction::Metabolism;
use crate::render::{render_start, render_update};
use crate::world::World;
use crate::{Load, Run};
use rand::rngs::SmallRng;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::{mpsc, Mutex};
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

impl Simulation {
    pub async fn run(
        &self,
        world: Arc<Mutex<World>>,
        small_rng: &mut SmallRng,
        world_info_tx: broadcast::Sender<WorldInfo>,
        client_command_rx: &mut mpsc::Receiver<ClientCommand>,
    ) -> Result<(), Box<dyn Error>> {
        if self.text_ui {
            render_start();
        }
        let mut tick: u64 = 0;
        let mut save_nr = 0;

        let frequencies = &self.frequencies;

        let render_world = Arc::clone(&world);

        tokio::spawn(async move {
            loop {
                let _ = world_info_tx.send(WorldInfo::new(&*render_world.lock().await));
                time::sleep(Duration::new(0, 1_000_000_000u32 / 8)).await;
            }
        });

        if self.dump {
            let save_world = Arc::clone(&world);
            tokio::spawn(async move {
                loop {
                    let result = save_file(&*save_world.lock().await, save_nr);
                    if result.is_err() {
                        println!("Could not write save file");
                        break;
                    }
                    save_nr += 1;
                    time::sleep(Duration::from_secs(60)).await;
                }
            });
        }

        if self.text_ui {
            let text_ui_world = Arc::clone(&world);
            tokio::spawn(async move {
                loop {
                    render_update();
                    println!("{}", text_ui_world.lock().await);
                    time::sleep(Duration::new(0, 1_000_000_000u32 / 8)).await;
                }
            });
        }

        loop {
            let mutate = tick % frequencies.mutation_frequency == 0;
            let receive_command = tick % frequencies.redraw_frequency == 0;

            world.lock().await.update(small_rng, self);
            if mutate {
                world
                    .lock()
                    .await
                    .mutate_memory(small_rng, self.memory_mutation_amount);
                world.lock().await.mutate_memory_insert(small_rng);
                // world.mutate_memory_delete(small_rng);
                world
                    .lock()
                    .await
                    .mutate_processor_stack(small_rng, self.processor_stack_mutation_amount)
            }

            if receive_command {
                if let Ok(cmd) = client_command_rx.try_recv() {
                    match cmd {
                        ClientCommand::Stop => loop {
                            // doesn't handle other commands while paused..
                            if let Some(cmd) = client_command_rx.recv().await {
                                match cmd {
                                    ClientCommand::Start => break,
                                    ClientCommand::Stop => {
                                        // no op when already stopped
                                    }
                                    ClientCommand::Disassemble { x, y, respond } => {
                                        respond
                                            .send(disassemble(&*world.lock().await, x, y))
                                            .unwrap();
                                    }
                                }
                            }
                        },
                        ClientCommand::Start => {
                            // no op when already started
                        }
                        ClientCommand::Disassemble { x, y, respond } => {
                            respond
                                .send(disassemble(&*world.lock().await, x, y))
                                .unwrap();
                        }
                    }
                }
                if let Ok(ClientCommand::Stop) = client_command_rx.try_recv() {
                    loop {
                        if let Some(ClientCommand::Start) = client_command_rx.recv().await {
                            break;
                        }
                    }
                }
            }
            tick = tick.wrapping_add(1);
        }
    }
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
