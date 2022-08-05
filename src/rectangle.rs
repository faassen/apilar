use rand::Rng;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Rectangle {
    pub fn random_coords(&self, rng: &mut rand::rngs::SmallRng) -> (usize, usize) {
        let x = rng.gen_range(self.x..self.x + self.w);
        let y = rng.gen_range(self.y..self.y + self.h);
        (x, y)
    }
}
