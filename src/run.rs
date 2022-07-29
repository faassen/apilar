use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::computer::Computer;
use crate::info::WorldInfo;
use crate::render::{render_start, render_update};
use crate::serve::serve;
use crate::world::World;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::error::Error;
use std::fs::File;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub async fn run(
    width: usize,
    height: usize,
    starting_memory_size: usize,
    starting_resources: u64,
    max_processors: usize,
    world_resources: u64,
    instructions_per_update: usize,
    mutation_frequency: u64,
    redraw_frequency: u64,
    save_frequency: u64,
    memory_mutation_amount: u64,
    processor_stack_mutation_amount: u64,
    eat_amount: u64,
    death_rate: u32,
    dump: bool,
    words: Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    let assembler = Assembler::new();

    let mut computer = Computer::new(starting_memory_size, max_processors, starting_resources);
    assembler.assemble_words(words, &mut computer.memory, 0);
    computer.add_processor(0);

    let mut world = World::new(width, height, eat_amount, world_resources);
    world.set((width / 2, height / 2), computer);

    let mut small_rng = SmallRng::from_entropy();

    let (world_info_tx, world_info_rx) = mpsc::channel(32);
    let (client_command_tx, mut client_command_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        serve(world_info_rx, client_command_tx).await;
    });

    simulation(
        &mut world,
        &mut small_rng,
        world_info_tx,
        &mut client_command_rx,
        instructions_per_update,
        mutation_frequency,
        redraw_frequency,
        save_frequency,
        memory_mutation_amount,
        processor_stack_mutation_amount,
        death_rate,
        dump,
    )
    .await
}

async fn simulation(
    world: &mut World,
    small_rng: &mut SmallRng,
    world_info_tx: mpsc::Sender<WorldInfo>,
    client_command_rx: &mut mpsc::Receiver<ClientCommand>,
    instructions_per_update: usize,
    mutation_frequency: u64,
    redraw_frequency: u64,
    save_frequency: u64,
    memory_mutation_amount: u64,
    processor_stack_mutation_amount: u64,
    death_rate: u32,
    dump: bool,
) -> Result<(), Box<dyn Error>> {
    render_start();
    let mut i: u64 = 0;
    let mut save_nr = 0;
    let mut started = true;

    loop {
        let redraw = i % redraw_frequency == 0;
        let mutate = i % mutation_frequency == 0;
        let save = i % save_frequency == 0;
        let receive_command = i % redraw_frequency == 0;

        if started {
            world.update(small_rng, instructions_per_update, death_rate);
            if mutate {
                world.mutate(
                    small_rng,
                    memory_mutation_amount,
                    processor_stack_mutation_amount,
                );
            }
            if save && dump {
                let file = File::create(format!("apilar-dump{}.cbor", save_nr))?;
                serde_cbor::to_writer(file, &world)?;
                save_nr += 1;
            }

            if redraw {
                // XXX does try send work?
                let _ = world_info_tx.try_send(WorldInfo::new(world)); // .await?;

                render_update();
                println!("{}", world);
            }
        }
        if receive_command {
            if let Ok(result) = client_command_rx.try_recv() {
                match result {
                    ClientCommand::Stop => loop {
                        if let Some(result) = client_command_rx.recv().await {
                            match result {
                                ClientCommand::Start => {
                                    started = true;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    },
                    ClientCommand::Start => {
                        // started = true;
                    }
                }
                sleep(Duration::from_millis(100)).await;
            }
        }
        i = i.wrapping_add(1);
    }
}
