use rand::rngs::SmallRng;

use crate::memory::Memory;
use crate::processor::Processor;

pub struct Computer {
    pub memory: Memory,
    max_processors: usize,
    processors: Vec<Processor>,
}

impl Computer {
    pub fn new(size: usize, max_processors: usize) -> Computer {
        Computer {
            memory: Memory::new(size),
            max_processors,
            processors: Vec::new(),
        }
    }

    pub fn execute(&mut self, rng: &mut SmallRng, amount_per_processor: usize) {
        // execute amount of instructions per processor
        for processor in &mut self.processors {
            processor.execute_amount(&mut self.memory, rng, amount_per_processor);
        }

        // obtain any start instructions
        let mut to_start: Vec<usize> = Vec::new();
        for processor in &self.processors {
            if let Some(address) = processor.get_start() {
                to_start.push(address);
            }
        }

        // sweep any dead processors
        // found in descriptoin of nightly drain_filter
        let mut i = 0;
        while i < self.processors.len() {
            if !self.processors[i].alive {
                self.processors.remove(i);
            } else {
                i += 1;
            }
        }

        // add new processors to start
        for address in to_start {
            if self.processors.len() < self.max_processors {
                self.processors.push(Processor::new(address));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;
    use crate::assembler::Assembler;

    fn execute(text: &str) -> (Processor, Memory, SmallRng) {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        let amount = assembler.assemble(text, &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);
        processor.execute_amount(&mut memory, &mut small_rng, amount);
        return (processor, memory, small_rng);
    }
}
