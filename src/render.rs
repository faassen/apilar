use crate::habitat::Habitat;
use std::fmt;

// display procedure based off https://oneorten.dev/blog/automata_rust_1/

impl fmt::Display for Habitat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.rows {
            for location in row.iter() {
                let ch = if location.computer.is_some() {
                    '#'
                } else if location.resources > 5000 {
                    'X'
                } else if location.resources > 2000 {
                    'x'
                } else if location.resources > 0 {
                    '.'
                } else {
                    ' '
                };

                write!(f, "{}", ch)?;
            }

            writeln!(f)?;
        }
        let (resources_free, resources_bound, resources_memory) = self.resources_amounts();
        let resources_total = resources_free + resources_bound + resources_memory;
        let computers_amount = self.computers_amount();
        let processors_amount = self.processors_amount();
        let processors_per_computer: f64 = processors_amount as f64 / computers_amount as f64;
        let resources_per_computer: f64 = resources_bound as f64 / computers_amount as f64;
        let memory_per_computer: f64 = resources_memory as f64 / computers_amount as f64;
        writeln!(f, "Computers : {:>8}", computers_amount)?;
        writeln!(f, "Processors: {:>8}", processors_amount)?;
        writeln!(f, "Proc per computer  : {:>8.3}", processors_per_computer)?;
        writeln!(f, "Bound per computer : {:>8.3}", resources_per_computer)?;
        writeln!(f, "Memory per computer: {:>8.3}", memory_per_computer)?;
        writeln!(
            f,
            "Resources Free: {:>10} Bound: {:>8} Memory: {:>8} Total {:>10}",
            resources_free, resources_bound, resources_memory, resources_total
        )?;
        Ok(())
    }
}

pub fn render_start() {
    print!("\x1b[2J\x1b[?25l");
}

pub fn render_update() {
    print!("\x1b[;H");
}
