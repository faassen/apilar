use serde_derive::{Deserialize, Serialize};

#[derive(
    Debug, Hash, Eq, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive, Serialize, Deserialize,
)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
