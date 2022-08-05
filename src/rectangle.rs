use rand::Rng;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rectangle {
    fn new(x: usize, y: usize, w: usize, h: usize) -> Rectangle {
        Rectangle { x, y, w, h }
    }

    pub fn random_coords(&self, rng: &mut rand::rngs::SmallRng) -> (usize, usize) {
        let x = rng.gen_range(self.x..self.x + self.w);
        let y = rng.gen_range(self.y..self.y + self.h);
        (x, y)
    }
}
