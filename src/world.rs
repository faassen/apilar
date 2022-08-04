use std::sync::{Arc, Mutex};

use crate::{
    habitat::{Habitat, HabitatConfig},
    ticks::Ticks,
};
use rand::rngs::SmallRng;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize)]
struct Rectangle {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Connection {
    from_rect: Rectangle,
    from_id: usize,
    to_rect: Rectangle,
    to_id: usize,
    transmit_frequency: u32, // use some kind of type, newtype? time-based?
}

#[derive(Debug, Serialize, Deserialize)]
struct Island {
    habitat: Habitat,
    config: HabitatConfig,
    connections: Vec<Connection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorldState {
    islands: Vec<Island>,
}

impl WorldState {
    pub fn update(&mut self, ticks: Ticks, rng: &mut SmallRng) {
        for island in &mut self.islands {
            let mutate = ticks.is_at(island.config.mutation_frequency);
            island.habitat.update(rng, &island.config);
            if mutate {
                island.habitat.mutate(rng, &island.config.mutation);
            }
        }
    }
}

// this should really become part of run.rs
#[derive(Debug)]
struct World {
    state: Arc<Mutex<WorldState>>,
}

const COMMAND_PROCESS_FREQUENCY: Ticks = Ticks(10000);

impl World {
    pub fn update(&self, rng: &mut SmallRng, mut main_loop_control_rx: mpsc::Receiver<bool>) {
        let mut ticks = Ticks(0);
        let receive_command = ticks.is_at(COMMAND_PROCESS_FREQUENCY);
        loop {
            self.state.lock().unwrap().update(ticks, rng);

            if receive_command {
                if let Ok(started) = main_loop_control_rx.try_recv() {
                    if !started {
                        while let Some(started) = main_loop_control_rx.blocking_recv() {
                            if started {
                                break;
                            }
                        }
                    }
                }
            }
            ticks = ticks.tick();
        }
    }
}
