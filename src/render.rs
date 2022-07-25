use crate::world::World;
use std::fmt;

// display procedure based off https://oneorten.dev/blog/automata_rust_1/

impl fmt::Display for World {
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

            write!(f, "\n")?;
        }
        let (resources_free, resources_bound, resources_memory) = self.resources_amounts();
        let resources_total = resources_free + resources_bound + resources_memory;
        write!(f, "Computers : {:>8}\n", self.computers_amount())?;
        write!(f, "Processors: {:>8}\n", self.processors_amount())?;
        write!(
            f,
            "Resources Free {:>12} Bound {:>10} Memory {:>10} Total {:>12}\n",
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
