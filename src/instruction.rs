// use num_derive::{FromPrimitive, ToPrimitive};
use rand::rngs::SmallRng;
use rand::Rng;
use strum_macros::{Display, EnumIter};

use crate::memory::Memory;
use crate::processor::Processor;

#[derive(EnumIter, Debug, PartialEq, Display, FromPrimitive, ToPrimitive)]
pub enum Instruction {
    // Noop
    NOOP = 0,
    // Numbers
    N1 = 1,
    N2,
    N4,
    N8,
    N16,
    N32,
    N64,
    N128,
    RND, // Random number

    // stack operators
    DUP = 20,
    DROP,
    SWAP,
    OVER,
    ROT,

    // Arithmetic
    ADD = 40,
    SUB,
    MUL,
    DIV,
    MOD,

    // Comparison
    EQ = 60,
    GT,
    LT,

    // Logic
    NOT = 80,
    AND,
    OR,

    // control
    JMP = 100, // also serves as return
    JMPIF,     // jump if boolean true,
    CALL,      // put return address on stack before jumping,
    CALLIF,    // call if boolean true

    // memory
    ADDR = 120,
    READ,
    WRITE,

    // processors
    START = 140, // start a new processor
    END,         // end this processor's existence
}

// execution system: spawn new processors until amount of processors is
// filled. Kill the oldest processor. This can be implemented with a
// wrap around index overwriting the oldest processor with a new one

impl Instruction {
    pub fn decode(value: u8) -> Option<Instruction> {
        num::FromPrimitive::from_u8(value)
    }

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
            Instruction::RND => {
                processor.push(rng.gen::<u64>());
            }
            Instruction::DUP => {
                processor.push(processor.top());
            }
            Instruction::DROP => {
                processor.drop();
            }
            Instruction::SWAP => {
                processor.swap();
            }
            Instruction::OVER => {
                processor.over();
            }
            Instruction::JMP => {
                let popped = processor.pop_address(memory);
                if let Some(address) = popped {
                    processor.ip = address;
                }
            }

            Instruction::ADDR => {
                processor.push(processor.ip as u64);
            }
            Instruction::READ => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_success() {
        assert_eq!(Instruction::decode(0), Some(Instruction::NOOP));
    }

    #[test]
    fn test_decode_failure() {
        assert_eq!(Instruction::decode(u8::MAX), None);
    }
}
