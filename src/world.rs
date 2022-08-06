use crate::assembler::{text_to_words, Assembler};
use crate::client_command::ClientCommand;
use crate::computer::Computer;
use crate::config::RunConfig;
use crate::habitat::Habitat;
use crate::info::WorldStateInfo;
use crate::island::{Connection, Island};
use crate::rectangle::Rectangle;
use crate::serve::serve_task;
use crate::ticks::Ticks;
use crate::topology::Topology;
use anyhow::Result;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::time;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldState {
    islands: Vec<Arc<Mutex<Island>>>,
    observed_island: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    world_state: Arc<Mutex<WorldState>>,
}

impl World {
    pub fn new(islands: Vec<Island>) -> World {
        World {
            world_state: Arc::new(Mutex::new(WorldState {
                islands: islands
                    .into_iter()
                    .map(|island| Arc::new(Mutex::new(island)))
                    .collect(),
                observed_island: 0,
            })),
        }
    }

    pub fn from_world_state(world_state: WorldState) -> World {
        World {
            world_state: Arc::new(Mutex::new(world_state)),
        }
    }

    pub fn set_observed(&mut self, island_id: usize) {
        self.world_state.lock().unwrap().set_observed(island_id);
    }

    pub fn get_observed(&self) -> Arc<Mutex<Island>> {
        self.world_state.lock().unwrap().get_observed()
    }

    pub fn run(&self, run_config: RunConfig, assembler: Assembler) -> Result<()> {
        let (habitat_info_tx, _) = broadcast::channel(32);
        let (client_command_tx, client_command_rx) = mpsc::channel(32);

        if run_config.server {
            tokio::spawn(serve_task(habitat_info_tx.clone(), client_command_tx));
        }

        tokio::spawn(Self::render_world_task(
            Arc::clone(&self.world_state),
            habitat_info_tx,
            run_config.redraw_frequency,
        ));

        tokio::spawn(Self::client_command_task(
            Arc::clone(&self.world_state),
            assembler,
            client_command_rx,
        ));

        if run_config.autosave.enabled {
            tokio::spawn(Self::save_world_task(
                Arc::clone(&self.world_state),
                run_config.autosave.frequency,
            ));
        }

        self.spawn_connection_tasks();

        let handles = self.spawn_island_tasks();
        for handle in handles {
            handle.join().unwrap();
        }
        Ok(())
    }

    pub fn spawn_island_tasks(&self) -> Vec<std::thread::JoinHandle<()>> {
        let mut handles = Vec::new();
        for island in &self.world_state.lock().unwrap().islands {
            let island = Arc::clone(island);
            let handle = std::thread::spawn(move || WorldState::island_task(island));
            handles.push(handle);
        }
        handles
    }

    fn spawn_connection_tasks(&self) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();
        let islands_amount = self.world_state.lock().unwrap().islands.len();
        for from_island_id in 0..islands_amount {
            let connections = self
                .world_state
                .lock()
                .unwrap()
                .get_connections(from_island_id);
            for connection in connections {
                let world_state = Arc::clone(&self.world_state);
                handles.push(tokio::spawn(Self::connection_task(
                    world_state,
                    from_island_id,
                    connection.from_rect.clone(),
                    connection.to_id,
                    connection.to_rect.clone(),
                    connection.transmit_frequency,
                )));
            }
        }
        handles
    }

    async fn render_world_task(
        world_state: Arc<Mutex<WorldState>>,
        tx: broadcast::Sender<WorldStateInfo>,
        duration: Duration,
    ) {
        loop {
            let _ = tx.send((&*world_state.lock().unwrap()).into());
            time::sleep(duration).await;
        }
    }

    async fn save_world_task(world_state: Arc<Mutex<WorldState>>, duration: Duration) {
        let mut save_nr = 0;
        loop {
            let result = world_state.lock().unwrap().save_world(save_nr);
            if result.is_err() {
                println!("Could not write save file");
                break;
            }
            save_nr += 1;
            time::sleep(duration).await;
        }
    }

    async fn client_command_task(
        world_state: Arc<Mutex<WorldState>>,
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
                    world_state.lock().unwrap().set_observed(island_id);
                }
                ClientCommand::Disassemble { x, y, respond } => {
                    let world = world_state.lock().unwrap();
                    let island = world.islands[world.observed_island].lock().unwrap();
                    respond
                        .send(island.habitat.disassemble(&assembler, x, y))
                        .unwrap();
                }
            }
        }
        Ok(())
    }

    async fn connection_task(
        world_state: Arc<Mutex<WorldState>>,
        from_island_id: usize,
        from_rect: Rectangle,
        to_island_id: usize,
        to_rect: Rectangle,
        duration: Duration,
    ) {
        let mut rng = SmallRng::from_entropy();
        loop {
            world_state.lock().unwrap().transfer(
                from_island_id,
                &from_rect,
                to_island_id,
                &to_rect,
                &mut rng,
            );
            time::sleep(duration).await;
        }
    }
}

impl WorldState {
    // this is the only task that isn't async but runs in a thread to make use of
    // multiple cores
    fn island_task(island: Arc<Mutex<Island>>) {
        let mut ticks = Ticks(0);
        let mut rng = SmallRng::from_entropy();
        loop {
            island.lock().unwrap().update(ticks, &mut rng);
            ticks = ticks.tick();
        }
    }

    fn save_world(&self, save_nr: u64) -> Result<()> {
        let file = BufWriter::new(File::create(format!("apilar-dump{:06}.aplr", save_nr))?);

        let mut zip = zip::ZipWriter::new(file);

        zip.start_file("data.cbor", zip::write::FileOptions::default())?;
        serde_cbor::to_writer(&mut zip, &self)?;
        zip.finish()?;
        Ok(())
    }

    fn set_observed(&mut self, island_id: usize) {
        if island_id >= self.islands.len() {
            println!("Island id {} out of range", island_id);
            return;
        }
        self.observed_island = island_id;
    }

    pub fn get_observed(&self) -> Arc<Mutex<Island>> {
        Arc::clone(&self.islands[self.observed_island])
    }

    pub fn get_observed_id(&self) -> usize {
        self.observed_island
    }

    pub fn get_islands(&self) -> &[Arc<Mutex<Island>>] {
        &self.islands
    }

    fn transfer(
        &mut self,
        from_island_id: usize,
        from_rect: &Rectangle,
        to_island_id: usize,
        to_rect: &Rectangle,
        rng: &mut SmallRng,
    ) {
        let islands = self.get_islands_by_id(&[from_island_id, to_island_id]);
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

    pub fn get_islands_by_id(&self, island_ids: &[usize]) -> Vec<Arc<Mutex<Island>>> {
        island_ids
            .iter()
            .map(|id| Arc::clone(&self.islands[*id]))
            .collect()
    }

    fn get_connections(&self, from_island_id: usize) -> Vec<Connection> {
        let from_island = &self.islands[from_island_id];
        // XXX clone is a bit of a hack
        return from_island.lock().unwrap().connections.clone();
    }
}

impl TryFrom<&Topology> for World {
    type Error = anyhow::Error;

    fn try_from(topology: &Topology) -> Result<Self> {
        let mut islands = Vec::new();
        for island_description in &topology.islands {
            let habitat = Habitat::new(
                island_description.width,
                island_description.height,
                island_description.resources,
            );
            // XXX should verify that connections make sense, both id and dimensions
            islands.push(Island::new(
                habitat,
                island_description.config.clone(),
                island_description.disaster.clone(),
                island_description.connections.clone(),
            ))
        }
        let assembler = Assembler::new();
        for computer_description in &topology.computers {
            let habitat = &mut islands[computer_description.island_id].habitat;
            let mut file = BufReader::new(File::open(computer_description.filename.clone())?);
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let words = text_to_words(&contents);
            let mut computer = Computer::new(words.len(), computer_description.resources);
            assembler.assemble_words(words, &mut computer.memory, 0);
            computer.add_processor(0);
            habitat.set((computer_description.x, computer_description.y), computer);
        }
        Ok(World::new(islands))
    }
}
