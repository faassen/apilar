use crate::assembler::{text_to_words, Assembler};
use crate::computer::Computer;
use crate::habitat::{Habitat, HabitatConfig};
use crate::island::Island;
use crate::ticks::Ticks;
use crate::topology::Topology;
use anyhow::Result;
use rand::rngs::SmallRng;
use serde_derive::{Deserialize, Serialize};

use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub islands: Vec<Island>,
    pub observed_island: usize,
}

impl World {
    pub fn new(islands: Vec<Island>) -> World {
        World {
            islands,
            observed_island: 0,
        }
    }

    pub fn from_island(island: Island) -> World {
        World {
            islands: vec![island],
            observed_island: 0,
        }
    }

    pub fn from_habitat(habitat: Habitat, config: HabitatConfig) -> World {
        Self::from_island(Island::new(habitat, config, Vec::new()))
    }

    pub fn update(&mut self, ticks: Ticks, rng: &mut SmallRng) {
        for island in &mut self.islands {
            island.update(ticks, rng);
        }
        // guard against allocation in the common case where no connection
        // needs to transfmit
        if self.needs_transmit(ticks) {
            self.update_connections(ticks, rng)
        }
    }

    pub fn update_connections(&mut self, ticks: Ticks, rng: &mut SmallRng) {
        let mut transfers = Vec::new();
        for from_id in 0..self.islands.len() {
            let island = &self.islands[from_id];
            for connection in &island.connections {
                // check again if we need to transmit
                let transmit = ticks.is_at(connection.transmit_frequency);
                if !transmit {
                    continue;
                };
                let transfer = island.habitat.get_connection_transfer(
                    rng,
                    &connection.from_rect,
                    &connection.to_rect,
                    &self.islands[connection.to_id].habitat,
                );
                if let Some(transfer) = transfer {
                    let (from_coords, to_coords, computer) = transfer;

                    transfers.push((from_id, from_coords, connection.to_id, to_coords, computer));
                }
            }
        }
        for transfer in transfers {
            let (from_id, from_coords, to_id, to_coords, computer) = transfer;
            let from_island = &mut self.islands[from_id];
            from_island.habitat.get_mut(from_coords).computer = None;
            let to_island = &mut self.islands[to_id];
            to_island.habitat.get_mut(to_coords).computer = Some(computer);
        }
    }

    pub fn needs_transmit(&self, ticks: Ticks) -> bool {
        for from_id in 0..self.islands.len() {
            let island = &self.islands[from_id];
            for connection in &island.connections {
                let transmit = ticks.is_at(connection.transmit_frequency);
                if transmit {
                    return true;
                }
            }
        }
        false
    }

    pub fn set_observed(&mut self, island_id: usize) {
        if island_id >= self.islands.len() {
            println!("Island id {} out of range", island_id);
            return;
        }
        self.observed_island = island_id;
    }

    pub fn habitat(&self) -> &Habitat {
        &self.islands[self.observed_island].habitat
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
