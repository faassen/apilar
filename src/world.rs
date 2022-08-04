use crate::island::{Island, IslandConfig};
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
struct IslandInfo {
    island: Island,
    config: IslandConfig,
    connections: Vec<Connection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct World {
    islands: Vec<IslandInfo>,
}
