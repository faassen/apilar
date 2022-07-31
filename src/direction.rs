use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive, Serialize, Deserialize)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
