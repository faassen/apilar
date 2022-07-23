use moveslice::Moveslice;
use rand::rngs::SmallRng;

use crate::instruction::Instruction;
use crate::memory::Memory;

const STACK_SIZE: usize = 64;

pub struct Processor {
    pub ip: usize,
    stack_pointer: usize,
    stack: [u64; STACK_SIZE],
}

impl Processor {
    pub fn new(ip: usize) -> Processor {
        return Processor {
            ip,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
        };
    }

    pub fn execute(&mut self, memory: &mut Memory, rng: &mut SmallRng) {
        let value = memory.values[self.ip];
        let instruction: Option<Instruction> = num::FromPrimitive::from_u8(value);
        match instruction {
            Some(instruction) => instruction.execute(self, memory, rng),
            None => {
                // no op, we cannot interpret this as a valid instruction
            }
        }
    }

    pub fn push(&mut self, value: u64) {
        if self.stack_pointer >= (STACK_SIZE - 1) {
            self.compact_stack();
        }
        self.stack[self.stack_pointer] = value;
        self.stack_pointer += 1;
    }

    pub fn pop(&mut self) -> u64 {
        if self.stack_pointer == 0 {
            return u64::MAX;
        }
        let result = self.stack[self.stack_pointer];
        self.stack_pointer -= 1;
        return result;
    }

    pub fn pop_address(&mut self, memory: &Memory) -> Option<usize> {
        if self.stack_pointer == 0 {
            return None;
        }
        let result = self.stack[self.stack_pointer] as usize;
        if result >= memory.values.len() {
            return None;
        }
        return Some(result);
    }

    pub fn top(&self) -> u64 {
        self.stack[self.stack_pointer]
    }

    pub fn drop(&mut self) {
        if self.stack_pointer == 0 {
            return;
        }
        self.stack_pointer -= 1;
    }

    pub fn swap(&mut self) {
        if self.stack_pointer <= 1 {
            return;
        }
        let first = self.stack_pointer - 1;
        let second = self.stack_pointer;
        let temp = self.stack[second];
        self.stack[second] = self.stack[first];
        self.stack[first] = temp;
    }

    pub fn compact_stack(&mut self) {
        self.stack_pointer = STACK_SIZE / 2;
        self.stack.moveslice(usize::from(self.stack_pointer).., 0);
    }
}
