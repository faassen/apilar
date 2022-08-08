use crate::instruction::Metabolism;
use crate::memory::Memory;
use crate::processor::Processor;
use crate::want::Wants;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Computer {
    pub resources: u64,
    pub memory: Memory,
    pub wants: Wants,
    pub processors: Vec<Processor>,
}

impl Computer {
    pub fn new(size: usize, resources: u64) -> Computer {
        Computer {
            resources,
            memory: Memory::new(size),
            wants: Wants::new(),
            processors: Vec::new(),
        }
    }

    pub fn split(&mut self, address: usize) -> Option<Computer> {
        // we cannot split off nothing
        if address == 0 || address >= self.memory.values.len() {
            return None;
        }

        let parent_memory_values = self.memory.values[..address].to_vec();
        let child_memory_values = self.memory.values[address..].to_vec();

        let mut parent_processors: Vec<Processor> = Vec::new();
        let mut child_processors: Vec<Processor> = Vec::new();

        for mut processor in self.processors.clone() {
            if processor.ip < address {
                processor.adjust_backward(address, child_memory_values.len());
                parent_processors.push(processor);
            } else {
                processor.adjust_backward(0, address);
                child_processors.push(processor);
            }
        }

        let child_resources = self.resources / 2;
        let parent_resources = self.resources - child_resources;

        self.resources = parent_resources;
        self.processors = parent_processors;
        self.memory = Memory::from_values(parent_memory_values);

        Some(Computer {
            resources: child_resources,
            memory: Memory::from_values(child_memory_values),
            wants: Wants::new(),
            processors: child_processors,
        })
    }

    pub fn merge(&mut self, other: &Computer, max_processors: usize) {
        for mut processor in other.processors.clone() {
            let distance = self.memory.values.len();
            processor.adjust_forward(0, distance);
            self.processors.push(processor);
        }
        self.memory.values.extend(other.memory.values.clone());
        if self.processors.len() > max_processors {
            // throw away any excess processors
            // this may lead to a strategy where being near max processors is good
            // for predation
            self.processors = self.processors[0..max_processors].to_vec();
        }
        self.resources += other.resources;
    }

    pub fn add_processor(&mut self, index: usize) {
        self.processors.push(Processor::new(index));
    }

    pub fn execute(
        &mut self,
        rng: &mut SmallRng,
        instructions_per_update: usize,
        max_processors: usize,
        metabolism: &Metabolism,
    ) {
        self.wants.clear();
        // execute amount of instructions per processor
        for processor in &mut self.processors {
            processor.execute_amount(
                &mut self.memory,
                &mut self.wants,
                rng,
                instructions_per_update,
                metabolism,
            );
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
        for address in self.wants.start.get() {
            if self.processors.len() < max_processors {
                self.processors.push(Processor::new(address));
            }
        }

        // grow memory if we want to grow
        if let Some(amount) = self.wants.grow.choose(rng) {
            let amount = if amount <= self.resources {
                amount
            } else {
                self.resources
            };
            for _ in 0..amount {
                self.memory.values.push(u8::MAX);
            }
            self.resources -= amount;
        }

        if let Some(amount) = self.wants.shrink.choose(rng) {
            let amount = amount as usize;
            let amount = if amount < self.memory.values.len() {
                amount
            } else {
                self.memory.values.len()
            };
            for _ in 0..amount {
                self.memory.values.pop();
            }
            // ensure that processors referring to the shrunken bit are fixed up
            for processor in &mut self.processors {
                processor.adjust_backward(self.memory.values.len(), amount);
            }
            self.resources += amount as u64;
        }
    }

    pub fn mutate_memory_overwrite(&mut self, rng: &mut SmallRng) {
        if self.memory.values.is_empty() {
            return;
        }
        let address = rng.gen_range(0..self.memory.values.len());
        self.memory.values[address] = rng.gen::<u8>();
    }

    pub fn mutate_memory_insert(&mut self, rng: &mut SmallRng) {
        if self.memory.values.is_empty() {
            return;
        }
        let address = rng.gen_range(0..self.memory.values.len());
        if self.resources > 0 {
            self.memory.values.insert(address, rng.gen::<u8>());
            self.resources -= 1;
            for processor in &mut self.processors {
                processor.adjust_forward(address, 1);
            }
        }
    }

    pub fn mutate_memory_delete(&mut self, rng: &mut SmallRng) {
        if self.memory.values.is_empty() {
            return;
        }
        let address = rng.gen_range(0..self.memory.values.len());
        if self.resources > 0 {
            self.memory.values.remove(address);
            self.resources += 1;
            for processor in &mut self.processors {
                processor.adjust_backward(address, 1);
            }
        }
    }

    pub fn mutate_processors(&mut self, rng: &mut SmallRng) {
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

        let mut computer = Computer::new(1024, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        let mut rng = SmallRng::from_seed([0; 32]);

        computer.add_processor(0);

        computer.execute(
            &mut rng,
            words_amount * 64,
            10,
            &Metabolism {
                max_eat_amount: 0,
                max_grow_amount: 0,
                max_shrink_amount: 0,
            },
        );

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

        let mut computer = Computer::new(4, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2);

        let splitted = computer.split(2).unwrap();
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
    fn test_split_at_start_doesnt_split() {
        let assembler = Assembler::new();

        let text = "
        N1
        N2
        N3
        N4
        ";
        let words = text_to_words(text);

        let mut computer = Computer::new(4, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2);

        let splitted = computer.split(0);
        assert!(splitted.is_none());

        assert_eq!(computer.memory.values, [1, 2, 3, 4]);
        assert_eq!(computer.resources, 100);
        assert_eq!(computer.processors.len(), 2);
    }

    #[test]
    fn test_split_beyond_end_doesnt_split() {
        let assembler = Assembler::new();

        let text = "
        N1
        N2
        N3
        N4
        ";
        let words = text_to_words(text);

        let mut computer = Computer::new(4, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2);

        let splitted = computer.split(4);
        assert!(splitted.is_none());

        assert_eq!(computer.memory.values, [1, 2, 3, 4]);
        assert_eq!(computer.resources, 100);
        assert_eq!(computer.processors.len(), 2);
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

        let mut computer = Computer::new(4, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.add_processor(2);

        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2); // oob

        computer.processors[2].set_current_head_value(0); // oob
        computer.processors[3].set_current_head_value(2);

        let splitted = computer.split(2).unwrap();
        assert_eq!(computer.memory.values, [1, 2]);
        assert_eq!(computer.resources, 50);
        assert_eq!(computer.processors.len(), 2);
        assert_eq!(computer.processors[0].ip, 0);
        assert_eq!(computer.processors[0].get_current_head_value(), Some(0));
        // oob gets reset to None
        assert_eq!(computer.processors[1].get_current_head_value(), None);

        assert_eq!(splitted.memory.values, [3, 4]);
        assert_eq!(splitted.resources, 50);
        assert_eq!(splitted.processors.len(), 2);
        assert_eq!(splitted.processors[0].ip, 0);
        // oob gets reset to None
        assert_eq!(splitted.processors[0].get_current_head_value(), None);
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

        let mut computer = Computer::new(5, 107);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2);

        let splitted = computer.split(2).unwrap();
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

        let mut computer = Computer::new(4, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(2);
        computer.processors[0].set_current_head_value(0);
        computer.processors[1].set_current_head_value(2);

        let splitted = computer.split(2).unwrap();
        computer.merge(&splitted, 10);

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

        let mut computer = Computer::new(4, 100);
        assembler.assemble_words(words.clone(), &mut computer.memory, 0);
        computer.add_processor(0);
        computer.add_processor(1);
        computer.add_processor(2);

        let mut splitted = computer.split(2).unwrap();
        splitted.add_processor(2);
        computer.merge(&splitted, 3);

        assert_eq!(computer.memory.values, [1, 2, 3, 4]);
        assert_eq!(computer.resources, 100);
        assert_eq!(computer.processors.len(), 3);
        assert_eq!(computer.processors[0].ip, 0);
        assert_eq!(computer.processors[1].ip, 1);
        assert_eq!(computer.processors[2].ip, 2);
        // fourth one is eliminated
    }
}
