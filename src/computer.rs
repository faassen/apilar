use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use crate::direction::Direction;
use crate::memory::Memory;
use crate::processor::Processor;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Computer {
    pub max_processors: usize,
    pub resources: u64,
    pub memory: Memory,
    pub processors: Vec<Processor>,
}

impl Computer {
    pub fn new(size: usize, max_processors: usize, resources: u64) -> Computer {
        Computer {
            max_processors,
            resources,
            memory: Memory::new(size),
            processors: Vec::new(),
        }
    }

    pub fn split(&mut self, address: usize) -> Computer {
        let parent_memory_values = self.memory.values[..address].to_vec();
        let child_memory_values = self.memory.values[address..].to_vec();

        let mut parent_processors: Vec<Processor> = Vec::new();
        let mut child_processors: Vec<Processor> = Vec::new();

        for mut processor in self.processors.clone() {
            if processor.ip < address {
                // reset heads that happen to be in the other half
                processor.fix_out_of_bounds_heads(address);
                parent_processors.push(processor);
            } else {
                processor.ip -= address;
                processor.shift_heads_backward(address);
                child_processors.push(processor);
            }
        }

        let child_resources = self.resources / 2;
        let parent_resources = self.resources - child_resources;

        self.resources = parent_resources;
        self.processors = parent_processors;
        self.memory = Memory::from_values(parent_memory_values);

        Computer {
            resources: child_resources,
            max_processors: self.max_processors,
            memory: Memory::from_values(child_memory_values),
            processors: child_processors,
        }
    }

    pub fn merge(&mut self, other: &Computer) {
        for mut processor in other.processors.clone() {
            let distance = self.memory.values.len();
            processor.ip += distance;
            processor.shift_heads_forward(distance);
            self.processors.push(processor);
        }
        self.memory.values.extend(other.memory.values.clone());
        if self.processors.len() > self.max_processors {
            // throw away any excess processors
            // this may lead to a strategy where being near max processors is good
            // for predation
            self.processors = self.processors[0..self.max_processors].to_vec();
        }
        self.resources += other.resources;
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
            if let Some(address) = processor.want_start {
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

        // grow memory if we want to grow
        if self.want_grow() && self.resources > 0 {
            self.memory.values.push(0);
            self.resources -= 1;
        }

        total
    }

    pub fn mutate_memory(&mut self, rng: &mut SmallRng, amount: u64) {
        if self.memory.values.is_empty() {
            return;
        }
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

    pub fn want_split(&self) -> Option<(Direction, usize)> {
        for processor in &self.processors {
            if let Some(want_split) = processor.want_split {
                return Some(want_split);
            }
        }
        None
    }

    pub fn want_merge(&self) -> Option<Direction> {
        for processor in &self.processors {
            if let Some(want_merge) = processor.want_merge {
                return Some(want_merge);
            }
        }
        None
    }

    pub fn want_eat(&self) -> bool {
        for processor in &self.processors {
            if processor.want_eat {
                return true;
            }
        }
        false
    }

    pub fn want_grow(&self) -> bool {
        for processor in &self.processors {
            if processor.want_grow {
                return true;
            }
        }
        false
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
        ADDR  # h0 = start
        N1
        HEAD
        N0
        COPY  # h1 = reader
        N2
        HEAD
        N0    
        COPY  # h2 = writer
        N8
        N8
        MUL
        DUP
        FORWARD # h2 forward 64
        DUP
        ADD # 128 on stack
        N3
        HEAD
        N2
        COPY  # h3 = h2, start offspring
        N4
        HEAD  
        ADDR  # h4 = loop
        N1
        HEAD
        READ  # read from h1
        N1
        FORWARD
        N2
        HEAD
        WRITE # write to h2
        N1
        FORWARD
        DUP   # duplicate 128 
        N3
        DISTANCE # distance h2 and h3
        SWAP
        LT    # if distance < 128
        N4
        HEAD
        JMPIF # jump to h4, loop
        N3
        HEAD
        START # start offspring at N3
        N0
        HEAD
        JMP   # jump to first addr
        ";
        let words = text_to_words(text);
        let words_amount = words.len();

        let mut computer = Computer::new(1024, 10, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        computer.add_processor(0);
        computer.execute(&mut small_rng, words_amount * 64);

        let disassembled =
            assembler.disassemble_to_words(&computer.memory.values[64..64 + words_amount]);

        assert_eq!(&disassembled, &words);
        // a new processor was spawned
        assert_eq!(computer.processors.len(), 2);
        assert_eq!(computer.processors[1].address(), 64);
    }

    #[test]
    fn test_split() {
        let assembler = Assembler::new();

        let text = "
        N1
        N2
        N3
        N4
        ";
        let words = text_to_words(text);

        let mut computer = Computer::new(4, 10, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2);

        let splitted = computer.split(2);
        assert_eq!(computer.memory.values, [1, 2]);
        assert_eq!(computer.resources, 50);
        assert_eq!(computer.processors.len(), 1);
        assert_eq!(computer.processors[0].ip, 0);
        assert_eq!(computer.processors[0].get_current_head_value(), Some(0));

        assert_eq!(splitted.memory.values, [3, 4]);
        assert_eq!(splitted.resources, 50);
        assert_eq!(splitted.processors.len(), 1);
        assert_eq!(splitted.processors[0].ip, 0);
        assert_eq!(splitted.processors[0].get_current_head_value(), Some(0));
    }

    #[test]
    fn test_split_out_of_bounds() {
        let assembler = Assembler::new();

        let text = "
        N1
        N2
        N3
        N4
        ";
        let words = text_to_words(text);

        let mut computer = Computer::new(4, 10, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.add_processor(2);

        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2); // oob

        computer.processors[2].set_current_head_value(0); // oob
        computer.processors[3].set_current_head_value(2);

        let splitted = computer.split(2);
        assert_eq!(computer.memory.values, [1, 2]);
        assert_eq!(computer.resources, 50);
        assert_eq!(computer.processors.len(), 2);
        assert_eq!(computer.processors[0].ip, 0);
        assert_eq!(computer.processors[0].get_current_head_value(), Some(0));
        // oob gets reset to 0
        assert_eq!(computer.processors[1].get_current_head_value(), Some(0));

        assert_eq!(splitted.memory.values, [3, 4]);
        assert_eq!(splitted.resources, 50);
        assert_eq!(splitted.processors.len(), 2);
        assert_eq!(splitted.processors[0].ip, 0);
        // oob gets reset to 0
        assert_eq!(splitted.processors[0].get_current_head_value(), Some(0));
        assert_eq!(splitted.processors[1].get_current_head_value(), Some(0));
    }

    #[test]
    fn test_split_uneven() {
        let assembler = Assembler::new();

        let text = "
        N1
        N2
        N3
        N4
        N5
        ";
        let words = text_to_words(text);

        let mut computer = Computer::new(5, 10, 107);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2);

        let splitted = computer.split(2);
        assert_eq!(computer.memory.values, [1, 2]);
        assert_eq!(computer.resources, 54);
        assert_eq!(computer.processors.len(), 1);
        assert_eq!(computer.processors[0].ip, 0);
        assert_eq!(computer.processors[0].get_current_head_value(), Some(0));

        assert_eq!(splitted.memory.values, [3, 4, 5]);
        assert_eq!(splitted.resources, 53);
        assert_eq!(splitted.processors.len(), 1);
        assert_eq!(splitted.processors[0].ip, 0);
        assert_eq!(splitted.processors[0].get_current_head_value(), Some(0));
    }

    #[test]
    fn test_merge() {
        let assembler = Assembler::new();

        let text = "
        N1
        N2
        N3
        N4
        ";
        let words = text_to_words(text);

        let mut computer = Computer::new(4, 10, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2);

        let splitted = computer.split(2);
        computer.merge(&splitted);

        assert_eq!(computer.memory.values, [1, 2, 3, 4]);
        assert_eq!(computer.resources, 100);
        assert_eq!(computer.processors.len(), 2);
        assert_eq!(computer.processors[0].ip, 0);
        assert_eq!(computer.processors[1].ip, 2);
        assert_eq!(computer.processors[0].get_current_head_value(), Some(0));
        assert_eq!(computer.processors[1].get_current_head_value(), Some(2));
    }

    #[test]
    fn test_merge_too_many_processors() {
        let assembler = Assembler::new();

        let text = "
        N1
        N2
        N3
        N4
        ";
        let words = text_to_words(text);

        let mut computer = Computer::new(4, 3, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(1);
        computer.add_processor(2);

        let mut splitted = computer.split(2);
        splitted.add_processor(2);
        computer.merge(&splitted);

        assert_eq!(computer.memory.values, [1, 2, 3, 4]);
        assert_eq!(computer.resources, 100);
        assert_eq!(computer.processors.len(), 3);
        assert_eq!(computer.processors[0].ip, 0);
        assert_eq!(computer.processors[1].ip, 1);
        assert_eq!(computer.processors[2].ip, 2);
        // fourth one is eliminated
    }
}
