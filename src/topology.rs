use crate::{
    habitat::{Death, HabitatConfig, Mutation},
    instruction::Metabolism,
    island::{Connection, Rectangle},
    ticks::Ticks,
};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct Topology {
    islands: Vec<IslandDescription>,
    computers: Vec<ComputerDescription>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IslandDescription {
    config: HabitatConfig,
    resources: u64,
    connections: Vec<Connection>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ComputerDescription {
    island_id: usize,
    code: String,
    x: usize,
    y: usize,
    resources: u64,
    memory_size: Option<usize>,
}

pub fn run() {
    let islands = vec![IslandDescription {
        config: HabitatConfig {
            instructions_per_update: 10,
            max_processors: 10,
            mutation_frequency: Ticks(100000),
            mutation: Mutation {
                overwrite_amount: 1,
                insert_amount: 1,
                delete_amount: 0,
                stack_amount: 1,
            },
            death: Death {
                rate: 10000,
                memory_size: 2usize.pow(13),
            },
            metabolism: Metabolism {
                max_eat_amount: 128,
                max_grow_amount: 16,
                max_shrink_amount: 16,
            },
        },
        resources: 400,
        connections: vec![Connection {
            from_id: 0,
            from_rect: Rectangle {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            to_id: 1,
            to_rect: Rectangle {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            transmit_frequency: Duration::new(1, 0),
        }],
    }];
    let topology = Topology {
        islands,
        computers: vec![],
    };

    println!("{}", serde_json::to_string_pretty(&topology).unwrap());
}
