use crate::assembler::{text_to_words, Assembler};
use crate::computer::Computer;
use crate::habitat::Habitat;
use crate::island::Island;
use crate::topology::Topology;
use anyhow::Result;

use serde_derive::{Deserialize, Serialize};

use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub islands: Vec<Arc<Mutex<Island>>>,
    pub observed_island: usize,
}

impl World {
    pub fn new(islands: Vec<Island>) -> World {
        World {
            islands: islands
                .into_iter()
                .map(|island| Arc::new(Mutex::new(island)))
                .collect(),
            observed_island: 0,
        }
    }

    pub fn get_islands(&self, island_ids: &[usize]) -> Vec<Arc<Mutex<Island>>> {
        island_ids
            .iter()
            .map(|id| Arc::clone(&self.islands[*id]))
            .collect()
    }

    pub fn set_observed(&mut self, island_id: usize) {
        if island_id >= self.islands.len() {
            println!("Island id {} out of range", island_id);
            return;
        }
        self.observed_island = island_id;
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
