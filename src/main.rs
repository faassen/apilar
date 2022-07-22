extern crate num;
#[macro_use]
extern crate num_derive;

use moveslice::Moveslice;
use rand::rngs::SmallRng;
use rand::Rng;

const STACK_SIZE: usize = 64;

#[derive(FromPrimitive, ToPrimitive)]
enum Instruction {
    // Numbers
    N0 = 0,
    N1,
    N2,
    N4,
    N8,
    N16,
    N32,
    N64,
    N128,
    Rnd, // Random number

    // stack operators
    Dup = 20,
    Drop,
    Swap,
    Over,
    Rot,

    // Arithmetic
    Add = 40,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq = 60,
    Gt,
    Lt,

    // Logic
    Not = 80,
    And,
    Or,

    // control
    Jmp = 100, // also serves as return
    JmpIf,     // jump if boolean true,
    Call,      // put return address on stack before jumping,
    CallIf,    // call if boolean true

    // memory
    Addr = 120,
    Read,
    Write,
}

impl Instruction {
    pub fn execute(&self, processor: &mut Processor, memory: &mut Memory, rng: &mut SmallRng) {
        match self {
            Instruction::N0 => {
                processor.push(0);
            }
            Instruction::N1 => {
                processor.push(1);
            }
            Instruction::N2 => {
                processor.push(2);
            }
            Instruction::N4 => {
                processor.push(4);
            }
            Instruction::N8 => {
                processor.push(8);
            }
            Instruction::N16 => {
                processor.push(16);
            }
            Instruction::N32 => {
                processor.push(32);
            }
            Instruction::N64 => {
                processor.push(64);
            }
            Instruction::N128 => {
                processor.push(128);
            }
            Instruction::Rnd => {
                processor.push(rng.gen::<u64>());
            }
            Instruction::Dup => {
                processor.push(processor.top());
            }
            Instruction::Drop => {
                processor.drop();
            }
            Instruction::Swap => {
                processor.swap();
            }

            Instruction::Jmp => {
                let popped = processor.pop_address(memory);
                if let Some(address) = popped {
                    processor.ip = address;
                }
            }

            Instruction::Addr => {
                processor.push(processor.ip as u64);
            }
            Instruction::Read => {
                let popped = processor.pop_address(memory);
                let value = match popped {
                    Some(address) => memory.values[address],
                    None => u64::MAX,
                };
                processor.push(value);
            }
        }
    }
}

struct Memory {
    values: Vec<u64>,
}

struct Processor {
    ip: usize,
    stack_pointer: usize,
    stack: [u64; STACK_SIZE],
}

impl Processor {
    pub fn new(ip: usize) -> Processor {
        return Processor {
            ip,
            stack: [0; 64],
            stack_pointer: 0,
        };
    }

    pub fn execute(&mut self, memory: &mut Memory, rng: &mut SmallRng) {
        let value = memory.values[self.ip];
        let instruction: Option<Instruction> = num::FromPrimitive::from_u64(value);
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

fn main() {
    println!("Hello, world!");
}
