// use num_derive::{FromPrimitive, ToPrimitive};
use rand::rngs::SmallRng;
use rand::Rng;
use strum_macros::{Display, EnumIter};

use crate::memory::Memory;
use crate::processor::Processor;

#[derive(EnumIter, Display, FromPrimitive, ToPrimitive)]
pub enum Instruction {
    // Noop
    Noop = 0,
    // Numbers
    N1 = 1,
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

    // processors
    Spawn = 140, // spawn a new processor
    End,         // end this processor's existence
}

// execution system: spawn new processors until amount of processors is
// filled. Kill the oldest processor. This can be implemented with a
// wrap around index overwriting the oldest processor with a new one

impl Instruction {
    pub fn execute(&self, processor: &mut Processor, memory: &mut Memory, rng: &mut SmallRng) {
        match self {
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
                    None => u8::MAX,
                };
                processor.push(value as u64);
            }
            _ => panic!("unsupported instruction"),
        }
    }
}
