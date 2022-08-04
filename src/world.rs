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
    islands: Vec<Island>,
}

impl World {
    pub fn new(islands: Vec<Island>) -> World {
        World { islands }
    }

    pub fn from_island(island: Island) -> World {
        World {
            islands: vec![island],
        }
    }

    pub fn from_habitat(habitat: Habitat, config: HabitatConfig) -> World {
        Self::from_island(Island::new(habitat, config, Vec::new()))
    }

    pub fn update(&mut self, ticks: Ticks, rng: &mut SmallRng) {
        for island in &mut self.islands {
            island.update(ticks, rng);
        }
    }

    pub fn habitat(&self) -> &Habitat {
        &self.islands[0].habitat
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
