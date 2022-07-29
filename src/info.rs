use crate::computer::Computer;
use crate::world::{Location, World};
use serde_derive::Serialize;

// info useful for the UI

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInfo {
    width: usize,
    height: usize,
    total_free_resources: u64,
    total_bound_resources: u64,
    total_memory_resources: u64,
    total_computers: u64,
    total_processors: u64,
    locations: Vec<Vec<LocationInfo>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationInfo {
    free_resources: u64,
    computer: Option<ComputerInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputerInfo {
    memory_size: usize,
    processors: usize,
    bound_resources: u64,
}

impl WorldInfo {
    pub fn new(world: &World) -> WorldInfo {
        let (total_free_resources, total_bound_resources, total_memory_resources) =
            world.resources_amounts();
        let total_computers = world.computers_amount();
        let total_processors = world.processors_amount();

        let mut locations: Vec<Vec<LocationInfo>> = Vec::new();

        for row in &world.rows {
            let mut row_locations: Vec<LocationInfo> = Vec::new();
            for location in row {
                row_locations.push(LocationInfo::new(location))
            }
            locations.push(row_locations);
        }

        WorldInfo {
            width: world.width,
            height: world.height,
            total_free_resources,
            total_bound_resources,
            total_memory_resources,
            total_computers,
            total_processors,
            locations,
        }
    }
}

impl LocationInfo {
    pub fn new(location: &Location) -> LocationInfo {
        let computer: Option<ComputerInfo> = location.computer.as_ref().map(ComputerInfo::new);
        LocationInfo {
            free_resources: location.resources,
            computer,
        }
    }
}

impl ComputerInfo {
    pub fn new(computer: &Computer) -> ComputerInfo {
        ComputerInfo {
            memory_size: computer.memory.values.len(),
            processors: computer.processors.len(),
            bound_resources: computer.resources,
        }
    }
}