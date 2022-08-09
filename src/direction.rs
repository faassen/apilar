use serde_derive::{Deserialize, Serialize};

#[derive(
    Debug,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Clone,
    Copy,
    FromPrimitive,
    ToPrimitive,
    Serialize,
    Deserialize,
)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn flip(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::North
    }
}
