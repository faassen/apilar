use crate::computer::Computer;
use crate::habitat::Location;
use crate::island::Island;
use crate::world::WorldState;
use serde_derive::Serialize;

// info useful for the UI

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldStateInfo {
    islands: Vec<IslandInfo>,
    observed_island_id: usize,
    locations: Vec<Vec<LocationInfo>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IslandInfo {
    width: usize,
    height: usize,
    total_free_resources: u64,
    total_bound_resources: u64,
    total_memory_resources: u64,
    total_computers: u64,
    total_processors: u64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocationInfo {
    free_resources: u64,
    computer: Option<ComputerInfo>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComputerInfo {
    memory_size: usize,
    processors: usize,
    bound_resources: u64,
}

impl From<&WorldState> for WorldStateInfo {
    fn from(world_state: &WorldState) -> WorldStateInfo {
        let mut islands = Vec::new();
        for island in world_state.get_islands() {
            islands.push((&*island.lock().unwrap()).into());
        }
        let mut locations: Vec<Vec<LocationInfo>> = Vec::new();

        let observed = world_state.get_observed();
        let island = observed.lock().unwrap();

        let habitat = &island.habitat;

        for row in &habitat.rows {
            let mut row_locations: Vec<LocationInfo> = Vec::new();
            for location in row {
                row_locations.push(location.into());
            }
            locations.push(row_locations);
        }

        WorldStateInfo {
            locations,
            islands,
            observed_island_id: world_state.get_observed_id(),
        }
    }
}

impl From<&Island> for IslandInfo {
    fn from(island: &Island) -> IslandInfo {
        let (total_free_resources, total_bound_resources, total_memory_resources) =
            island.habitat.resources_amounts();
        let total_computers = island.habitat.computers_amount();
        let total_processors = island.habitat.processors_amount();

        let mut locations: Vec<Vec<LocationInfo>> = Vec::new();

        for row in &island.habitat.rows {
            let mut row_locations: Vec<LocationInfo> = Vec::new();
            for location in row {
                row_locations.push(location.into())
            }
            locations.push(row_locations);
        }

        IslandInfo {
            width: island.habitat.width,
            height: island.habitat.height,
            total_free_resources,
            total_bound_resources,
            total_memory_resources,
            total_computers,
            total_processors,
        }
    }
}

impl From<&Location> for LocationInfo {
    fn from(location: &Location) -> LocationInfo {
        let computer: Option<ComputerInfo> =
            location.computer.as_ref().map(|computer| computer.into());
        LocationInfo {
            free_resources: location.resources,
            computer,
        }
    }
}

impl From<&Computer> for ComputerInfo {
    fn from(computer: &Computer) -> ComputerInfo {
        ComputerInfo {
            memory_size: computer.memory.values.len(),
            processors: computer.processors.len(),
            bound_resources: computer.resources,
        }
    }
}
