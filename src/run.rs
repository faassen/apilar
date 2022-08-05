use crate::assembler::Assembler;
use crate::client_command::ClientCommand;
use crate::config::RunConfig;
use crate::habitat::Habitat;
use crate::info::WorldInfo;
use crate::island::Connection;
use crate::island::Island;
use crate::rectangle::Rectangle;
use crate::serve::serve_task;
use crate::ticks::Ticks;
use crate::topology::Topology;
use crate::world::World;
use crate::RunConfigArgs;
use anyhow::anyhow;
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

pub async fn load_command(cli: &RunConfigArgs) -> Result<()> {
    let file = BufReader::new(File::open(cli.filename.clone())?);

    let mut archive = zip::ZipArchive::new(file).unwrap();

    let cbor_file = match archive.by_name("data.cbor") {
        Ok(file) => file,
        Err(_) => {
            return Err(anyhow!("data.cbor not found in archive"));
        }
    };

    let run_config: RunConfig = RunConfig::from(cli);
    let world: World = serde_cbor::from_reader(cbor_file)?;

    let assembler = Assembler::new();

    run(run_config, Arc::new(Mutex::new(world)), assembler).await
}

pub async fn run_command(cli: &RunConfigArgs) -> Result<()> {
    let file = BufReader::new(File::open(cli.filename.clone())?);
    let topology: Topology = serde_json::from_reader(file)?;
    let world = World::try_from(&topology)?;
    let run_config = RunConfig::from(cli);
    let assembler = Assembler::new();
    run(run_config, Arc::new(Mutex::new(world)), assembler).await
}

async fn run(run_config: RunConfig, world: Arc<Mutex<World>>, assembler: Assembler) -> Result<()> {
    let (habitat_info_tx, _) = broadcast::channel(32);
    let (client_command_tx, client_command_rx) = mpsc::channel(32);

    if run_config.server {
        tokio::spawn(serve_task(habitat_info_tx.clone(), client_command_tx));
    }

    tokio::spawn(render_world_task(
        Arc::clone(&world),
        habitat_info_tx,
        run_config.redraw_frequency,
    ));

    tokio::spawn(client_command_task(
        Arc::clone(&world),
        assembler,
        client_command_rx,
    ));

    if run_config.autosave.enabled {
        tokio::spawn(save_world_task(
            Arc::clone(&world),
            run_config.autosave.frequency,
        ));
    }

    let mut handles = spawn_connection_tasks(Arc::clone(&world));
    handles.extend(spawn_islands(Arc::clone(&world)));
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}

fn spawn_islands(world: Arc<Mutex<World>>) -> Vec<std::thread::JoinHandle<()>> {
    let mut handles = Vec::new();
    for island in &Arc::clone(&world).lock().unwrap().islands {
        let island = Arc::clone(island);
        let handle = std::thread::spawn(move || island_simulation_task(island));
        handles.push(handle);
    }
    handles
}

fn spawn_connection_tasks(world: Arc<Mutex<World>>) -> Vec<std::thread::JoinHandle<()>> {
    let mut handles = Vec::new();
    let islands_amount = world.lock().unwrap().islands.len();
    for from_island_id in 0..islands_amount {
        let connections = get_connections(Arc::clone(&world), from_island_id);
        for connection in connections {
            let world = Arc::clone(&world);
            let handle = std::thread::spawn(move || {
                connection_task(
                    world,
                    from_island_id,
                    &connection.from_rect,
                    connection.to_id,
                    &connection.to_rect,
                    connection.transmit_frequency,
                )
            });
            handles.push(handle);
        }
    }
    handles
}

fn get_connections(world: Arc<Mutex<World>>, from_island_id: usize) -> Vec<Connection> {
    let from_island = &world.lock().unwrap().islands[from_island_id];
    // XXX clone is a bit of a hack
    return from_island.lock().unwrap().connections.clone();
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

async fn client_command_task(
    world: Arc<Mutex<World>>,
    assembler: Assembler,
    mut rx: mpsc::Receiver<ClientCommand>,
) -> Result<()> {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            ClientCommand::Stop => {
                // tx.send(false).await;
            }
            ClientCommand::Start => {
                // tx.send(true).await;
            }
            ClientCommand::Observe { island_id } => {
                world.lock().unwrap().set_observed(island_id);
            }
            ClientCommand::Disassemble { x, y, respond } => {
                let world = world.lock().unwrap();
                let island = world.islands[world.observed_island].lock().unwrap();
                respond
                    .send(disassemble(&island.habitat, &assembler, x, y))
                    .unwrap();
            }
        }
    }
    Ok(())
}

fn island_simulation_task(island: Arc<Mutex<Island>>) {
    let mut ticks = Ticks(0);
    let mut rng = SmallRng::from_entropy();
    loop {
        island.lock().unwrap().update(ticks, &mut rng);
        ticks = ticks.tick();
    }
}

fn connection_task(
    world: Arc<Mutex<World>>,
    from_island_id: usize,
    from_rect: &Rectangle,
    to_island_id: usize,
    to_rect: &Rectangle,
    duration: Duration,
) {
    let mut rng = SmallRng::from_entropy();
    loop {
        transfer(
            Arc::clone(&world),
            from_island_id,
            from_rect,
            to_island_id,
            to_rect,
            &mut rng,
        );
        std::thread::sleep(duration);
    }
}

fn transfer(
    world: Arc<Mutex<World>>,
    from_island_id: usize,
    from_rect: &Rectangle,
    to_island_id: usize,
    to_rect: &Rectangle,
    rng: &mut SmallRng,
) {
    let islands = &world
        .lock()
        .unwrap()
        .get_islands(&[from_island_id, to_island_id]);
    let from_island = &islands[0];
    let to_island = &islands[1];
    let transfer = from_island.lock().unwrap().habitat.get_connection_transfer(
        rng,
        from_rect,
        to_rect,
        &to_island.lock().unwrap().habitat,
    );
    if let Some((from_coords, to_coords, computer)) = transfer {
        from_island
            .lock()
            .unwrap()
            .habitat
            .get_mut(from_coords)
            .computer = None;
        to_island
            .lock()
            .unwrap()
            .habitat
            .get_mut(to_coords)
            .computer = Some(computer)
    }
}

fn save_world(world: &World, save_nr: u64) -> Result<()> {
    let file = BufWriter::new(File::create(format!("apilar-dump{:06}.aplr", save_nr))?);

    let mut zip = zip::ZipWriter::new(file);

    zip.start_file("data.cbor", zip::write::FileOptions::default())?;
    serde_cbor::to_writer(&mut zip, &world)?;
    zip.finish()?;
    Ok(())
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
