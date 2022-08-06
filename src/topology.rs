use crate::{
    habitat::HabitatConfig,
    island::{Connection, Disaster},
};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Topology {
    pub islands: Vec<IslandDescription>,
    pub computers: Vec<ComputerDescription>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IslandDescription {
    pub config: HabitatConfig,
    pub width: usize,
    pub height: usize,
    pub resources: u64,
    pub disaster: Option<Disaster>,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComputerDescription {
    pub island_id: usize,
    pub filename: String,
    pub x: usize,
    pub y: usize,
    pub resources: u64,
    pub memory_size: Option<usize>,
}
