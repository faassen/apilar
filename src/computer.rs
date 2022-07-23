use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::memory::Memory;
use crate::processor::Processor;

pub struct Computer {
    pub memory: Memory,
    max_processors: usize,
    pub processors: Vec<Processor>,
}

impl Computer {
    pub fn new(size: usize, max_processors: usize) -> Computer {
        Computer {
            memory: Memory::new(size),
            max_processors,
            processors: Vec::new(),
        }
    }

    pub fn add_processor(&mut self, index: usize) {
        self.processors.push(Processor::new(index));
    }

    pub fn execute(&mut self, rng: &mut SmallRng, amount_per_processor: usize) -> usize {
        // execute amount of instructions per processor
        let mut total = 0;
        for processor in &mut self.processors {
            total += processor.execute_amount(&mut self.memory, rng, amount_per_processor);
        }

        // obtain any start instructions
        let mut to_start: Vec<usize> = Vec::new();
        for processor in &self.processors {
            if let Some(address) = processor.get_start() {
                to_start.push(address);
            }
        }

        // sweep any dead processors
        // found in description of drain_filter (method in nightly)
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

        return total;
    }

    pub fn mutate_memory(&mut self, rng: &mut SmallRng, amount: u64) {
        for _ in 0..amount {
            let address = rng.gen_range(0..self.memory.values.len());
            self.memory.values[address] = rng.gen::<u8>();
        }
    }

    pub fn mutate_processors(&mut self, rng: &mut SmallRng, amount: u64) {
        for _ in 0..amount {
            let choice = self.processors.choose_mut(rng);
            if let Some(processor) = choice {
                if rng.gen_ratio(1, 5) {
                    processor.pop();
                } else {
                    processor.push(rng.gen::<u8>() as u64);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::{text_to_words, Assembler};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn test_replicate() {
        let assembler = Assembler::new();

        let text = "
        ADDR  # c
        DUP   # preserve starting point
        ADDR  # c loop
        SWAP  # loop c
        DUP   # loop c c
        READ  # loop c inst
        SWAP  # loop inst c
        DUP   # loop inst c c
        N8
        N8
        MUL
        ADD   # loop inst c c+64
        ROT   # loop c c+64 inst
        WRITE # loop c
        N1
        ADD   # loop c+1
        DUP   # loop c+1 c+1
        ADDR
        N8
        N8
        N4
        ADD
        ADD   # add to get end of replicator
        ADD   # loop c+1 c+1 end
        LT    # loop c+1 b
        ROT   # c+1 b loop
        SWAP  # c+1 loop b
        JMPIF # go to loop
        DROP  # start
        DUP   # start start
        N8
        N8
        MUL
        ADD   # start newstart
        START # start
        JMP   # jump to first addr
        ";
        let words = text_to_words(text);
        let words_amount = words.len();

        let mut computer = Computer::new(1024, 10);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        computer.add_processor(0);
        computer.execute(&mut small_rng, words_amount * words_amount);

        let disassembled =
            assembler.disassemble_to_words(&computer.memory.values[64..64 + words_amount]);

        assert_eq!(&disassembled, &words);
        // a new processor was spawned
        assert_eq!(computer.processors.len(), 2);
        assert_eq!(computer.processors[1].address(), 64);
    }
}
