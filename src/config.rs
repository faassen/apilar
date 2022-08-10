use crate::command::Autosave;
use crate::RunConfigArgs;
use std::time::Duration;

#[derive(Debug)]
pub struct RunConfig {
    pub autosave: Autosave,
    // how many milliseconds between redraws
    pub redraw_frequency: Duration,
    pub server: bool,
}

impl From<&RunConfigArgs> for RunConfig {
    fn from(cli: &RunConfigArgs) -> Self {
        RunConfig {
            autosave: Autosave {
                enabled: cli.autosave,
                // how many seconds between autosaves
                frequency: Duration::from_secs(cli.autosave_frequency),
            },
            redraw_frequency: Duration::from_millis(cli.redraw_frequency),
            server: !cli.no_server,
        }
    }
}
