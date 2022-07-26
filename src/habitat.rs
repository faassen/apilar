use crate::assembler::Assembler;
use crate::direction::Direction;
use crate::instruction::Metabolism;
use crate::rectangle::Rectangle;
use crate::{computer::Computer, ticks::Ticks};
use rand::rngs::SmallRng;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

const CONNECTION_SAMPLING_TRIES: u64 = 2u64.pow(5);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub resources: u64,
    pub computer: Option<Computer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habitat {
    pub width: usize,
    pub height: usize,
    pub rows: Vec<Vec<Location>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mutation {
    pub overwrite_amount: u64,
    pub insert_amount: u64,
    pub delete_amount: u64,
    pub stack_amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Death {
    pub rate: u32,
    pub memory_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatConfig {
    pub instructions_per_update: usize,
    pub max_processors: usize,
    // how many ticks between mutations
    pub mutation_frequency: Ticks,
    pub mutation: Mutation,
    pub death: Death,
    pub metabolism: Metabolism,
}

pub type Coords = (usize, usize);

impl Habitat {
    pub fn new(width: usize, height: usize, resources: u64) -> Habitat {
        let mut rows: Vec<Vec<Location>> = Vec::new();
        for _ in 0..height {
            let mut column_vec: Vec<Location> = Vec::new();
            for _ in 0..width {
                column_vec.push(Location::new(resources));
            }
            rows.push(column_vec);
        }
        Habitat {
            width,
            height,
            rows,
        }
    }

    fn neighbor_coords(&self, coords: Coords, direction: Direction) -> Coords {
        let (x, y) = coords;
        let ix = x as i32;
        let iy = y as i32;
        let (nx, ny): (i32, i32) = match direction {
            Direction::North => (ix, iy - 1),
            Direction::East => (ix + 1, iy),
            Direction::South => (ix, iy + 1),
            Direction::West => (ix - 1, iy),
        };
        let rx = nx.rem_euclid(self.width as i32);
        let ry = ny.rem_euclid(self.height as i32);
        (rx as usize, ry as usize)
    }

    pub fn set(&mut self, (x, y): Coords, computer: Computer) {
        self.rows[y][x].computer = Some(computer);
    }

    pub fn get(&self, (x, y): Coords) -> &Location {
        &self.rows[y][x]
    }

    pub fn get_mut(&mut self, (x, y): Coords) -> &mut Location {
        &mut self.rows[y][x]
    }

    pub fn get_random_coords(&self, rng: &mut SmallRng) -> Coords {
        let x = rng.gen_range(0..self.width);
        let y = rng.gen_range(0..self.height);
        (x, y)
    }

    pub fn is_empty(&self, coords: Coords) -> bool {
        self.get(coords).computer.is_none()
    }

    pub fn update(&mut self, rng: &mut SmallRng, config: &HabitatConfig) {
        let coords = self.get_random_coords(rng);

        let location = self.get_mut(coords);
        location.update(rng, config);

        if self.death(rng, coords, &config.death) {
            return;
        }

        if let Some((neighbor_coords, address)) = self.want_split(coords, rng) {
            self.split(coords, neighbor_coords, address);
        }
        if let Some(neighbor_coords) = self.want_merge(coords, rng) {
            let neighbor_computer = self.get(neighbor_coords).computer.clone();
            if let Some(neighbor_computer) = neighbor_computer {
                self.merge(
                    coords,
                    neighbor_coords,
                    &neighbor_computer,
                    config.max_processors,
                );
            }
        }
        if let Some(neighbor_coords) = self.want_move(coords, rng) {
            self.move_to(coords, neighbor_coords);
        }

        if let Some(amount) = self.want_eat(coords, rng) {
            self.eat(coords, amount);
        }
    }

    pub fn mutate(&mut self, rng: &mut SmallRng, mutation: &Mutation) {
        self.mutate_memory_overwrite(rng, mutation.overwrite_amount);
        self.mutate_memory_insert(rng, mutation.insert_amount);
        self.mutate_memory_delete(rng, mutation.delete_amount);
        self.mutate_processor_stack(rng, mutation.stack_amount)
    }

    pub fn mutate_memory_overwrite(&mut self, rng: &mut SmallRng, amount: u64) {
        for _ in 0..amount {
            let coords = self.get_random_coords(rng);
            let location = self.get_mut(coords);
            if let Some(computer) = &mut location.computer {
                computer.mutate_memory_overwrite(rng);
            }
        }
    }

    pub fn mutate_memory_insert(&mut self, rng: &mut SmallRng, amount: u64) {
        for _ in 0..amount {
            let coords = self.get_random_coords(rng);
            let location = self.get_mut(coords);
            if let Some(computer) = &mut location.computer {
                computer.mutate_memory_insert(rng);
            }
        }
    }

    pub fn mutate_memory_delete(&mut self, rng: &mut SmallRng, amount: u64) {
        for _ in 0..amount {
            let coords = self.get_random_coords(rng);
            let location = self.get_mut(coords);
            if let Some(computer) = &mut location.computer {
                computer.mutate_memory_delete(rng);
            }
        }
    }

    pub fn mutate_processor_stack(&mut self, rng: &mut SmallRng, amount: u64) {
        for _ in 0..amount {
            let coords = self.get_random_coords(rng);
            let location = self.get_mut(coords);
            if let Some(computer) = &mut location.computer {
                computer.mutate_processors(rng);
            }
        }
    }

    pub fn death(&mut self, rng: &mut SmallRng, coords: Coords, death: &Death) -> bool {
        let location = self.get_mut(coords);
        if let Some(computer) = &mut location.computer {
            if rng.gen_ratio(1, death.rate) || computer.memory.values.len() > death.memory_size {
                location.resources += computer.resources + computer.memory.values.len() as u64;
                location.computer = None;
                return true;
            }
        }
        false
    }

    pub fn die(&mut self, coords: Coords) {
        let location = self.get_mut(coords);
        if let Some(computer) = &mut location.computer {
            location.resources += computer.resources + computer.memory.values.len() as u64;
            location.computer = None;
        }
    }

    pub fn wipeout(&mut self, rng: &mut SmallRng, width: usize, height: usize) {
        let start_x = rng.gen_range(0..self.width);
        let start_y = rng.gen_range(0..self.height);
        for y in start_y..start_y + height {
            for x in start_x..start_x + width {
                let rx = x.rem_euclid(self.width);
                let ry = y.rem_euclid(self.height);
                self.die((rx, ry));
            }
        }
    }

    fn want_split(&self, coords: Coords, rng: &mut SmallRng) -> Option<(Coords, usize)> {
        let location = self.get(coords);
        if let Some(computer) = &location.computer {
            if let Some((direction, address)) = computer.wants.split.choose(rng) {
                let neighbor_coords = self.neighbor_coords(coords, direction);
                if self.is_empty(neighbor_coords) {
                    return Some((neighbor_coords, address));
                }
            }
        }
        None
    }

    fn want_merge(&self, coords: Coords, rng: &mut SmallRng) -> Option<Coords> {
        let location = self.get(coords);
        if let Some(computer) = &location.computer {
            if let Some((direction, strength)) = computer.wants.merge.choose_with_strength(rng) {
                let neighbor_coords = self.neighbor_coords(coords, direction);
                if !self.is_empty(neighbor_coords) {
                    let neighbor_computer = &self.get(neighbor_coords).computer;
                    if let Some(neighbor_computer) = neighbor_computer {
                        if let Some(block_strength) = neighbor_computer
                            .wants
                            .block_merge
                            .get_strength_by_value(direction.flip())
                        {
                            if strength > block_strength {
                                return Some(neighbor_coords);
                            } else {
                                return None;
                            }
                        } else {
                            return Some(neighbor_coords);
                        }
                    }
                }
            }
        }
        None
    }

    fn want_move(&self, coords: Coords, rng: &mut SmallRng) -> Option<Coords> {
        let location = self.get(coords);
        if let Some(computer) = &location.computer {
            if let Some(direction) = computer.wants.move_.choose(rng) {
                let neighbor_coords = self.neighbor_coords(coords, direction);
                if self.is_empty(neighbor_coords) {
                    return Some(neighbor_coords);
                }
            }
        }
        None
    }

    fn want_eat(&self, coords: Coords, rng: &mut SmallRng) -> Option<u64> {
        if let Some(computer) = &self.get(coords).computer {
            return computer.wants.eat.choose(rng);
        }
        None
    }

    fn split(&mut self, coords: Coords, neighbor_coords: Coords, address: usize) {
        let computer = &mut self.get_mut(coords).computer;
        if let Some(computer) = computer {
            let splitted = computer.split(address);
            let neighbor_location = self.get_mut(neighbor_coords);
            neighbor_location.computer = splitted;
        }
    }

    fn merge(
        &mut self,
        coords: Coords,
        neighbor_coords: Coords,
        neighbor_computer: &Computer,
        max_processors: usize,
    ) {
        let computer = &mut self.get_mut(coords).computer;
        if let Some(computer) = computer {
            computer.merge(neighbor_computer, max_processors);
        }
        let neighbor_location = self.get_mut(neighbor_coords);
        neighbor_location.computer = None;
    }

    fn move_to(&mut self, coords: Coords, neighbor_coords: Coords) {
        let computer = &self.get_mut(coords).computer;
        if let Some(computer) = computer {
            // ugh, expensive to move around
            let copy = computer.clone();
            let neighbor_location = self.get_mut(neighbor_coords);
            neighbor_location.computer = Some(copy);
        }
        let location = self.get_mut(coords);
        location.computer = None;
    }

    fn eat(&mut self, coords: Coords, eat_amount: u64) {
        let location = self.get_mut(coords);
        if let Some(computer) = &mut location.computer {
            let amount = if location.resources >= eat_amount {
                eat_amount
            } else {
                location.resources
            };
            computer.resources += amount;
            location.resources -= amount;
        }
    }

    pub fn computers_amount(&self) -> u64 {
        let mut total = 0;
        for row in &self.rows {
            for location in row {
                if location.computer.is_some() {
                    total += 1;
                }
            }
        }
        total
    }

    pub fn processors_amount(&self) -> u64 {
        let mut total = 0;
        for row in &self.rows {
            for location in row {
                if let Some(computer) = &location.computer {
                    total += computer.processors.len() as u64;
                }
            }
        }
        total
    }

    pub fn resources_amounts(&self) -> (u64, u64, u64) {
        let mut free: u64 = 0;
        let mut bound: u64 = 0;
        let mut memory: u64 = 0;

        for row in &self.rows {
            for location in row {
                free += location.resources;
                if let Some(computer) = &location.computer {
                    bound += computer.resources;
                    memory += computer.memory.values.len() as u64;
                }
            }
        }
        (free, bound, memory)
    }

    pub fn take_sample(
        &self,
        rng: &mut SmallRng,
        from_rect: &Rectangle,
        max_tries: u64,
    ) -> Option<(Computer, Coords)> {
        for _ in 0..max_tries {
            let coords = from_rect.random_coords(rng);
            if !self.is_empty(coords) {
                let location = self.get(coords);
                if let Some(computer) = &location.computer {
                    return Some((computer.clone(), coords));
                }
            }
        }
        None
    }

    pub fn get_place_sample_coords(
        &self,
        rng: &mut SmallRng,
        to_rect: &Rectangle,
        max_tries: u64,
    ) -> Option<(usize, usize)> {
        for _ in 0..max_tries {
            let coords = to_rect.random_coords(rng);
            if self.is_empty(coords) {
                return Some(coords);
            }
        }
        None
    }

    pub fn get_connection_transfer(
        &self,
        rng: &mut SmallRng,
        from_rect: &Rectangle,
        to_rect: &Rectangle,
        destination: &Habitat,
    ) -> Option<(Coords, Coords, Computer)> {
        let destination_coords =
            destination.get_place_sample_coords(rng, to_rect, CONNECTION_SAMPLING_TRIES);
        if let Some(destination_coords) = destination_coords {
            let computer = self.take_sample(rng, from_rect, CONNECTION_SAMPLING_TRIES);
            if let Some((computer, from_coords)) = computer {
                return Some((from_coords, destination_coords, computer));
            }
        }
        None
    }

    pub fn disassemble(&self, assembler: &Assembler, x: usize, y: usize) -> Result<String, String> {
        if x >= self.width {
            return Err("x out of range".to_string());
        }
        if y >= self.height {
            return Err("y out of range".to_string());
        }

        let location = self.get((x, y));
        if let Some(computer) = &location.computer {
            Ok(assembler.line_disassemble(&computer.memory.values))
        } else {
            Err("no computer".to_string())
        }
    }
}

impl Location {
    pub fn new(resources: u64) -> Location {
        Location {
            resources,
            computer: None,
        }
    }

    pub fn update(&mut self, rng: &mut SmallRng, config: &HabitatConfig) {
        let mut eliminate_computer: bool = false;

        if let Some(computer) = &mut self.computer {
            if computer.processors.is_empty() {
                self.resources += computer.resources + computer.memory.values.len() as u64;
                eliminate_computer = true;
            } else {
                computer.execute(
                    rng,
                    config.instructions_per_update,
                    config.max_processors,
                    &config.metabolism,
                );
            }
        }
        if eliminate_computer {
            self.computer = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_neighbor_out_of_bounds() {
        let habitat = Habitat::new(5, 5, 5);
        assert_eq!(habitat.neighbor_coords((2, 2), Direction::North), (2, 1));
        assert_eq!(habitat.neighbor_coords((2, 2), Direction::South), (2, 3));
        assert_eq!(habitat.neighbor_coords((2, 2), Direction::West), (1, 2));
        assert_eq!(habitat.neighbor_coords((2, 2), Direction::East), (3, 2));

        assert_eq!(habitat.neighbor_coords((1, 0), Direction::North), (1, 4));
        assert_eq!(habitat.neighbor_coords((1, 4), Direction::South), (1, 0));
        assert_eq!(habitat.neighbor_coords((0, 2), Direction::West), (4, 2));
        assert_eq!(habitat.neighbor_coords((4, 2), Direction::East), (0, 2));
    }

    #[test]
    fn test_want_split_no_obstructions() {
        let mut habitat = Habitat::new(5, 5, 5);
        let mut location = habitat.get_mut((2, 2));
        location.computer = Some(Computer::new(1, 1));
        if let Some(computer) = &mut location.computer {
            computer.wants.split.want((Direction::North, 1));
        }
        let mut rng = SmallRng::from_seed([0; 32]);
        let result = habitat.want_split((2, 2), &mut rng);
        assert_eq!(result, Some(((2, 1), 1)));
    }

    #[test]
    fn test_want_split_with_obstruction() {
        let mut habitat = Habitat::new(5, 5, 5);
        let mut location = habitat.get_mut((2, 2));
        location.computer = Some(Computer::new(1, 1));
        if let Some(computer) = &mut location.computer {
            computer.wants.split.want((Direction::North, 1));
        }
        let mut location_north = habitat.get_mut((2, 1));
        location_north.computer = Some(Computer::new(1, 1));

        let mut rng = SmallRng::from_seed([0; 32]);
        assert!(habitat.want_split((2, 2), &mut rng).is_none());
    }

    #[test]
    fn test_want_merge_no_neighbor() {
        let mut habitat = Habitat::new(5, 5, 5);
        let mut location = habitat.get_mut((2, 2));
        location.computer = Some(Computer::new(1, 1));
        if let Some(computer) = &mut location.computer {
            computer.wants.merge.want(Direction::North);
        }
        let mut rng = SmallRng::from_seed([0; 32]);
        assert!(habitat.want_merge((2, 2), &mut rng).is_none());
    }

    #[test]
    fn test_want_merge_with_unblocked_neighbor() {
        let mut habitat = Habitat::new(5, 5, 5);
        let mut location = habitat.get_mut((2, 2));
        location.computer = Some(Computer::new(1, 1));
        if let Some(computer) = &mut location.computer {
            computer.wants.merge.want(Direction::North);
        }
        let mut location_north = habitat.get_mut((2, 1));
        location_north.computer = Some(Computer::new(1, 1));

        let mut rng = SmallRng::from_seed([0; 32]);
        assert_eq!(habitat.want_merge((2, 2), &mut rng), Some((2, 1)));
    }

    #[test]
    fn test_want_merge_with_blocked_neighbor() {
        let mut habitat = Habitat::new(5, 5, 5);
        let mut location = habitat.get_mut((2, 2));
        location.computer = Some(Computer::new(1, 1));
        if let Some(computer) = &mut location.computer {
            computer.wants.merge.want(Direction::North);
        }

        let mut location_north = habitat.get_mut((2, 1));
        location_north.computer = Some(Computer::new(1, 1));
        let location_north = habitat.get_mut((2, 1));
        if let Some(computer) = &mut location_north.computer {
            computer.wants.block_merge.want(Direction::South);
        }

        let mut rng = SmallRng::from_seed([0; 32]);
        assert_eq!(habitat.want_merge((2, 2), &mut rng), None);
    }

    #[test]
    fn test_want_merge_overcome_blocked_neighbor() {
        let mut habitat = Habitat::new(5, 5, 5);
        let mut location = habitat.get_mut((2, 2));
        location.computer = Some(Computer::new(1, 1));
        if let Some(computer) = &mut location.computer {
            computer.wants.merge.want(Direction::North);
            computer.wants.merge.want(Direction::North);
        }

        let mut location_north = habitat.get_mut((2, 1));
        location_north.computer = Some(Computer::new(1, 1));
        let location_north = habitat.get_mut((2, 1));
        if let Some(computer) = &mut location_north.computer {
            computer.wants.block_merge.want(Direction::South);
        }

        let mut rng = SmallRng::from_seed([0; 32]);
        assert_eq!(habitat.want_merge((2, 2), &mut rng), Some((2, 1)));
    }

    #[test]
    fn test_want_merge_with_blocked_neighbor_but_not_in_right_direction() {
        let mut habitat = Habitat::new(5, 5, 5);
        let mut location = habitat.get_mut((2, 2));
        location.computer = Some(Computer::new(1, 1));
        if let Some(computer) = &mut location.computer {
            computer.wants.merge.want(Direction::North);
        }

        let mut location_north = habitat.get_mut((2, 1));
        location_north.computer = Some(Computer::new(1, 1));
        let location_north = habitat.get_mut((2, 1));
        if let Some(computer) = &mut location_north.computer {
            computer.wants.block_merge.want(Direction::West); // wrong direction
        }

        let mut rng = SmallRng::from_seed([0; 32]);
        assert_eq!(habitat.want_merge((2, 2), &mut rng), Some((2, 1)));
    }
}
