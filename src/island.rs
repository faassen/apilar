use crate::rectangle::Rectangle;
use crate::{
    habitat::{Habitat, HabitatConfig},
    ticks::Ticks,
};
use rand::rngs::SmallRng;
use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;
use std::time::Duration;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_rect: Rectangle,
    pub to_rect: Rectangle,
    pub to_id: usize,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub transmit_frequency: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Disaster {
    pub frequency: Ticks,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Island {
    pub habitat: Habitat,
    config: HabitatConfig,
    pub disaster: Option<Disaster>,
    pub connections: Vec<Connection>,
}

impl Island {
    pub fn new(
        habitat: Habitat,
        config: HabitatConfig,
        disaster: Option<Disaster>,
        connections: Vec<Connection>,
    ) -> Island {
        Island {
            habitat,
            config,
            disaster,
            connections,
        }
    }

    pub fn update(&mut self, ticks: Ticks, rng: &mut SmallRng) {
        self.habitat.update(rng, &self.config);

        let mutate = ticks.is_at(self.config.mutation_frequency);
        if mutate {
            self.habitat.mutate(rng, &self.config.mutation);
        }
        if let Some(disaster) = &self.disaster {
            let have_disaster = ticks.is_at(disaster.frequency);
            if have_disaster {
                self.habitat.wipeout(rng, disaster.width, disaster.height);
            }
        }
    }
}
