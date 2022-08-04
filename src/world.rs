use crate::habitat::{Habitat, HabitatConfig};
use crate::island::Island;
use crate::ticks::Ticks;
use rand::rngs::SmallRng;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    islands: Vec<Island>,
}

impl World {
    pub fn from_island(island: Island) -> World {
        World {
            islands: vec![island],
        }
    }

    pub fn from_habitat(habitat: Habitat, config: HabitatConfig) -> World {
        Self::from_island(Island::new(habitat, config))
    }

    pub fn update(&mut self, ticks: Ticks, rng: &mut SmallRng) {
        for island in &mut self.islands {
            island.update(ticks, rng);
        }
    }
}

// // this should really become part of run.rs
// #[derive(Debug)]
// struct World {
//     state: Arc<Mutex<WorldState>>,
// }

// const COMMAND_PROCESS_FREQUENCY: Ticks = Ticks(10000);

// impl World {
//     pub fn update(&self, rng: &mut SmallRng, mut main_loop_control_rx: mpsc::Receiver<bool>) {
//         let mut ticks = Ticks(0);
//         let receive_command = ticks.is_at(COMMAND_PROCESS_FREQUENCY);
//         loop {
//             self.state.lock().unwrap().update(ticks, rng);

//             if receive_command {
//                 if let Ok(started) = main_loop_control_rx.try_recv() {
//                     if !started {
//                         while let Some(started) = main_loop_control_rx.blocking_recv() {
//                             if started {
//                                 break;
//                             }
//                         }
//                     }
//                 }
//             }
//             ticks = ticks.tick();
//         }
//     }
// }
