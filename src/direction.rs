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

impl Default for Direction {
    fn default() -> Self {
        Direction::North
    }
}
