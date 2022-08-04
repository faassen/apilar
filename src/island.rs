use crate::{
    habitat::{Habitat, HabitatConfig},
    ticks::Ticks,
};
use rand::rngs::SmallRng;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Rectangle {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Connection {
    from_rect: Rectangle,
    from_id: usize,
    to_rect: Rectangle,
    to_id: usize,
    transmit_frequency: u32, // use some kind of type, newtype? time-based?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Island {
    habitat: Habitat,
    config: HabitatConfig,
    connections: Vec<Connection>,
}

impl Island {
    pub fn new(habitat: Habitat, config: HabitatConfig) -> Island {
        Island {
            habitat,
            config,
            connections: Vec::new(),
        }
    }

    pub fn update(&mut self, ticks: Ticks, rng: &mut SmallRng) {
        let mutate = ticks.is_at(self.config.mutation_frequency);
        self.habitat.update(rng, &self.config);
        if mutate {
            self.habitat.mutate(rng, &self.config.mutation);
        }
    }
}
