use crate::habitat::{Death, HabitatConfig, Mutation};
use crate::instruction::Metabolism;
use crate::run::Autosave;
use crate::{Load, Run};
use std::time::Duration;

pub struct Config {
    pub habitat_config: HabitatConfig,
    pub run_config: RunConfig,
}

pub struct RunConfig {
    pub autosave: Autosave,
    // how many milliseconds between redraws
    pub redraw_frequency: Duration,
    pub text_ui: bool,
    pub server: bool,
}

impl From<&Run> for Config {
    fn from(cli: &Run) -> Self {
        Config {
            habitat_config: HabitatConfig {
                instructions_per_update: cli.instructions_per_update,
                mutation_frequency: cli.mutation_frequency,
                mutation: Mutation {
                    overwrite_amount: cli.memory_overwrite_mutation_amount,
                    insert_amount: cli.memory_insert_mutation_amount,
                    delete_amount: cli.memory_delete_mutation_amount,
                    stack_amount: cli.processor_stack_mutation_amount,
                },
                death: Death {
                    rate: cli.death_rate,
                    memory_size: cli.death_memory_size,
                },
                metabolism: Metabolism {
                    max_eat_amount: cli.max_eat_amount,
                    max_grow_amount: cli.max_grow_amount,
                    max_shrink_amount: cli.max_shrink_amount,
                },
            },
            run_config: RunConfig {
                autosave: Autosave {
                    enabled: cli.autosave,
                    // how many milliseconds between autosaves
                    frequency: Duration::from_millis(cli.save_frequency),
                },
                redraw_frequency: Duration::from_millis(cli.redraw_frequency),
                text_ui: cli.text_ui,
                server: !cli.no_server,
            },
        }
    }
}

impl From<&Load> for Config {
    fn from(cli: &Load) -> Self {
        Config {
            habitat_config: HabitatConfig {
                instructions_per_update: cli.instructions_per_update,
                mutation_frequency: cli.mutation_frequency,
                mutation: Mutation {
                    overwrite_amount: cli.memory_overwrite_mutation_amount,
                    insert_amount: cli.memory_insert_mutation_amount,
                    delete_amount: cli.memory_delete_mutation_amount,
                    stack_amount: cli.processor_stack_mutation_amount,
                },
                death: Death {
                    rate: cli.death_rate,
                    memory_size: cli.death_memory_size,
                },
                metabolism: Metabolism {
                    max_eat_amount: cli.max_eat_amount,
                    max_grow_amount: cli.max_grow_amount,
                    max_shrink_amount: cli.max_shrink_amount,
                },
            },
            run_config: RunConfig {
                autosave: Autosave {
                    enabled: cli.autosave,
                    // how many milliseconds between autosaves
                    frequency: Duration::from_millis(cli.autosave_frequency),
                },
                redraw_frequency: Duration::from_millis(cli.redraw_frequency),
                text_ui: cli.text_ui,
                server: !cli.no_server,
            },
        }
    }
}
