use crate::client_command::ClientCommand;
use crate::info::WorldInfo;
use crate::render::{render_start, render_update};
use crate::world::World;
use rand::rngs::SmallRng;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use tokio::sync::mpsc;

pub struct Frequencies {
    pub mutation_frequency: u64,
    pub redraw_frequency: u64,
    pub save_frequency: u64,
}

pub struct Simulation {
    instructions_per_update: usize,
    memory_mutation_amount: u64,
    processor_stack_mutation_amount: u64,
    death_rate: u32,
    frequencies: Frequencies,
    dump: bool,
    text_ui: bool,
}

impl Simulation {
    pub fn new(
        instructions_per_update: usize,
        memory_mutation_amount: u64,
        processor_stack_mutation_amount: u64,
        death_rate: u32,
        frequencies: Frequencies,
        dump: bool,
        text_ui: bool,
    ) -> Simulation {
        Simulation {
            instructions_per_update,
            memory_mutation_amount,
            processor_stack_mutation_amount,
            death_rate,
            frequencies,
            dump,
            text_ui,
        }
    }

    pub async fn run(
        &self,
        world: &mut World,
        small_rng: &mut SmallRng,
        world_info_tx: mpsc::Sender<WorldInfo>,
        client_command_rx: &mut mpsc::Receiver<ClientCommand>,
    ) -> Result<(), Box<dyn Error>> {
        if self.text_ui {
            render_start();
        }
        let mut i: u64 = 0;
        let mut save_nr = 0;

        let frequencies = &self.frequencies;

        loop {
            let redraw = i % frequencies.redraw_frequency == 0;
            let mutate = i % frequencies.mutation_frequency == 0;
            let save = i % frequencies.save_frequency == 0;
            let receive_command = i % frequencies.redraw_frequency == 0;

            world.update(small_rng, self.instructions_per_update, self.death_rate);
            if mutate {
                world.mutate(
                    small_rng,
                    self.memory_mutation_amount,
                    self.processor_stack_mutation_amount,
                );
            }
            if save && self.dump {
                let file = BufWriter::new(File::create(format!("apilar-dump{:06}.cbor", save_nr))?);
                serde_cbor::to_writer(file, &world)?;
                save_nr += 1;
            }

            if self.text_ui && redraw {
                render_update();
                println!("{}", world);
            }

            if redraw {
                // XXX does try send work?
                let _ = world_info_tx.try_send(WorldInfo::new(world)); // .await?;
            }

            if receive_command {
                if let Ok(ClientCommand::Stop) = client_command_rx.try_recv() {
                    loop {
                        if let Some(ClientCommand::Start) = client_command_rx.recv().await {
                            break;
                        }
                    }
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
