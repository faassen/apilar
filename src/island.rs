use crate::{
    habitat::{Habitat, HabitatConfig},
    ticks::Ticks,
};
use rand::rngs::SmallRng;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rectangle {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub from_rect: Rectangle,
    pub from_id: usize,
    pub to_rect: Rectangle,
    pub to_id: usize,
    pub transmit_frequency: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Island {
    pub habitat: Habitat,
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
