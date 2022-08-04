use crate::habitat::{Habitat, HabitatConfig};
use crate::island::Island;
use crate::ticks::Ticks;
use rand::rngs::SmallRng;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    islands: Vec<Island>,
}

impl World {
    pub fn from_island(island: Island) -> World {
        World {
            islands: vec![island],
        }
    }

    pub fn from_habitat(habitat: Habitat, config: HabitatConfig) -> World {
        Self::from_island(Island::new(habitat, config))
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
