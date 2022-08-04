use crate::habitat::{Habitat, HabitatConfig};
use crate::island::Island;
use crate::ticks::Ticks;
use rand::rngs::SmallRng;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    habitats: Vec<Habitat>,
    islands: Vec<Island>,
}

impl World {
    pub fn from_habitat(habitat: Habitat, config: HabitatConfig) -> World {
        World {
            habitats: vec![habitat],
            islands: vec![Island::new(config)],
        }
    }

    pub fn update_habitat(&mut self, id: usize, ticks: Ticks, rng: &mut SmallRng) {
        let config = &self.islands[id].config;
        let habitat = &mut self.habitats[id];

        habitat.update(rng, config);
        let mutate = ticks.is_at(config.mutation_frequency);
        if mutate {
            habitat.mutate(rng, &config.mutation);
        }
    }

    pub fn update(&mut self, ticks: Ticks, rng: &mut SmallRng) {
        for id in 0..self.habitats.len() {
            self.update_habitat(id, ticks, rng)
        }
    }

    pub fn habitat(&self) -> &Habitat {
        &self.habitats[0]
    }
}
