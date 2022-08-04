use crate::instruction::Metabolism;
use crate::run::Autosave;
use crate::ticks::Ticks;
use crate::world::{Death, Mutation};
use crate::{Load, Run};
use std::time::Duration;

pub struct Frequencies {
    // how many milliseconds between redraws
    pub redraw_frequency: Duration,
}

pub struct Simulation {
    pub instructions_per_update: usize,
    // how many ticks between mutations
    pub mutation_frequency: Ticks,
    pub mutation: Mutation,
    pub death: Death,
    pub metabolism: Metabolism,
    pub frequencies: Frequencies,
    pub autosave: Autosave,
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
            death: Death {
                death_rate: cli.death_rate,
                death_memory_size: cli.death_memory_size,
            },
            metabolism: Metabolism {
                max_eat_amount: cli.max_eat_amount,
                max_grow_amount: cli.max_grow_amount,
                max_shrink_amount: cli.max_shrink_amount,
            },
            frequencies: Frequencies {
                redraw_frequency: Duration::from_millis(cli.redraw_frequency),
            },
            autosave: Autosave {
                autosave: cli.autosave,
                // how many milliseconds between autosaves
                autosave_frequency: Duration::from_millis(cli.save_frequency),
            },
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
            death: Death {
                death_rate: cli.death_rate,
                death_memory_size: cli.death_memory_size,
            },
            metabolism: Metabolism {
                max_eat_amount: cli.max_eat_amount,
                max_grow_amount: cli.max_grow_amount,
                max_shrink_amount: cli.max_shrink_amount,
            },
            frequencies: Frequencies {
                redraw_frequency: Duration::from_millis(cli.redraw_frequency),
            },
            autosave: Autosave {
                autosave: cli.autosave,
                // how many milliseconds between autosaves
                autosave_frequency: Duration::from_millis(cli.autosave_frequency),
            },
            text_ui: cli.text_ui,
        }
    }
}
