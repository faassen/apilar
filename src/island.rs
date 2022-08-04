use crate::{habitat::HabitatConfig, ticks::Ticks};
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
    pub config: HabitatConfig,
    connections: Vec<Connection>,
}

impl Island {
    pub fn new(config: HabitatConfig) -> Island {
        Island {
            config,
            connections: Vec::new(),
        }
    }
}
