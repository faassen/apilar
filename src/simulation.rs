use crate::instruction::Metabolism;
use crate::ticks::Ticks;
use crate::{Load, Run};
use std::time::Duration;

pub struct Frequencies {
    // how many ticks before mutations happen
    pub mutation_frequency: Ticks,
    // how many milliseconds between redraws
    pub redraw_frequency: Duration,
    // how many milliseconds between saves
    pub save_frequency: Duration,
}

pub struct Simulation {
    pub instructions_per_update: usize,
    pub memory_mutation_amount: u64,
    pub processor_stack_mutation_amount: u64,
    pub death_rate: u32,
    pub metabolism: Metabolism,
    pub frequencies: Frequencies,
    pub dump: bool,
    pub text_ui: bool,
}

impl From<&Run> for Simulation {
    fn from(run: &Run) -> Self {
        Simulation {
            instructions_per_update: run.instructions_per_update,
            memory_mutation_amount: run.memory_mutation_amount,
            processor_stack_mutation_amount: run.processor_stack_mutation_amount,
            death_rate: run.death_rate,
            metabolism: Metabolism {
                max_eat_amount: run.max_eat_amount,
                max_grow_amount: run.max_grow_amount,
                max_shrink_amount: run.max_shrink_amount,
            },
            frequencies: Frequencies {
                mutation_frequency: run.mutation_frequency,
                redraw_frequency: Duration::from_millis(run.redraw_frequency),
                save_frequency: Duration::from_millis(run.save_frequency),
            },
            dump: run.dump,
            text_ui: run.text_ui,
        }
    }
}

impl From<&Load> for Simulation {
    fn from(load: &Load) -> Self {
        Simulation {
            instructions_per_update: load.instructions_per_update,
            memory_mutation_amount: load.memory_mutation_amount,
            processor_stack_mutation_amount: load.processor_stack_mutation_amount,
            death_rate: load.death_rate,
            metabolism: Metabolism {
                max_eat_amount: load.max_eat_amount,
                max_grow_amount: load.max_grow_amount,
                max_shrink_amount: load.max_shrink_amount,
            },
            frequencies: Frequencies {
                mutation_frequency: load.mutation_frequency,
                redraw_frequency: Duration::from_millis(load.redraw_frequency),
                save_frequency: Duration::from_millis(load.save_frequency),
            },
            dump: load.dump,
            text_ui: load.text_ui,
        }
    }
}
