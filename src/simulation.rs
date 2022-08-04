use crate::instruction::Metabolism;
use crate::ticks::Ticks;
use crate::world::Mutation;
use crate::{Load, Run};
use std::time::Duration;

pub struct Frequencies {
    // how many milliseconds between redraws
    pub redraw_frequency: Duration,
    // how many milliseconds between saves
    pub save_frequency: Duration,
}

pub struct Simulation {
    pub instructions_per_update: usize,
    // how many ticks between mutations
    pub mutation_frequency: Ticks,
    pub mutation: Mutation,
    pub death_rate: u32,
    pub metabolism: Metabolism,
    pub frequencies: Frequencies,
    pub dump: bool,
    pub text_ui: bool,
}

impl From<&Run> for Simulation {
    fn from(cli: &Run) -> Self {
        Simulation {
            instructions_per_update: cli.instructions_per_update,
            mutation_frequency: cli.mutation_frequency,
            mutation: Mutation {
                memory_overwrite_mutation_amount: cli.memory_overwrite_mutation_amount,
                memory_insert_mutation_amount: cli.memory_insert_mutation_amount,
                memory_delete_mutation_amount: cli.memory_delete_mutation_amount,
                processor_stack_mutation_amount: cli.processor_stack_mutation_amount,
            },
            death_rate: cli.death_rate,
            metabolism: Metabolism {
                max_eat_amount: cli.max_eat_amount,
                max_grow_amount: cli.max_grow_amount,
                max_shrink_amount: cli.max_shrink_amount,
            },
            frequencies: Frequencies {
                redraw_frequency: Duration::from_millis(cli.redraw_frequency),
                save_frequency: Duration::from_millis(cli.save_frequency),
            },
            dump: cli.dump,
            text_ui: cli.text_ui,
        }
    }
}

impl From<&Load> for Simulation {
    fn from(cli: &Load) -> Self {
        Simulation {
            instructions_per_update: cli.instructions_per_update,
            mutation_frequency: cli.mutation_frequency,
            mutation: Mutation {
                memory_overwrite_mutation_amount: cli.memory_overwrite_mutation_amount,
                memory_insert_mutation_amount: cli.memory_insert_mutation_amount,
                memory_delete_mutation_amount: cli.memory_delete_mutation_amount,
                processor_stack_mutation_amount: cli.processor_stack_mutation_amount,
            },
            death_rate: cli.death_rate,
            metabolism: Metabolism {
                max_eat_amount: cli.max_eat_amount,
                max_grow_amount: cli.max_grow_amount,
                max_shrink_amount: cli.max_shrink_amount,
            },
            frequencies: Frequencies {
                redraw_frequency: Duration::from_millis(cli.redraw_frequency),
                save_frequency: Duration::from_millis(cli.save_frequency),
            },
            dump: cli.dump,
            text_ui: cli.text_ui,
        }
    }
}
