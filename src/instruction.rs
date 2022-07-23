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
            Instruction::NOOP => {
                // nothing
            }
            // Numbers
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

            // Stack manipulation
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
            Instruction::ROT => {
                processor.rot();
            }

            // Arithmetic
            Instruction::ADD => {
                let a = processor.pop();
                let b = processor.pop();
                processor.push(b.wrapping_add(a));
            }
            Instruction::SUB => {
                let a = processor.pop();
                let b = processor.pop();
                processor.push(b.wrapping_sub(a));
            }
            Instruction::MUL => {
                let a = processor.pop();
                let b = processor.pop();
                processor.push(b.wrapping_mul(a));
            }
            Instruction::DIV => {
                let a = processor.pop();
                let b = processor.pop();
                if a == 0 {
                    processor.push(0);
                    return;
                }
                processor.push(b.wrapping_div(a));
            }
            Instruction::MOD => {
                let a = processor.pop();
                let b = processor.pop();
                if a == 0 {
                    processor.push(0);
                    return;
                }
                processor.push(b.wrapping_rem(a));
            }

            // Comparison
            Instruction::EQ => {
                let a = processor.pop();
                let b = processor.pop();
                if a == b {
                    processor.push(1);
                } else {
                    processor.push(0);
                }
            }
            Instruction::GT => {
                let a = processor.pop();
                let b = processor.pop();
                if b > a {
                    processor.push(1);
                } else {
                    processor.push(0);
                }
            }
            Instruction::LT => {
                let a = processor.pop();
                let b = processor.pop();
                if b < a {
                    processor.push(1);
                } else {
                    processor.push(0);
                }
            }

            // Logic
            Instruction::NOT => {
                let a = processor.pop();
                if a > 0 {
                    processor.push(0);
                } else {
                    processor.push(1);
                }
            }
            Instruction::AND => {
                let a = processor.pop();
                let b = processor.pop();
                if a > 0 && b > 0 {
                    processor.push(1);
                } else {
                    processor.push(0);
                }
            }
            Instruction::OR => {
                let a = processor.pop();
                let b = processor.pop();
                if a > 0 || b > 0 {
                    processor.push(1);
                } else {
                    processor.push(0);
                }
            }

            // Control
            Instruction::JMP => {
                let popped = processor.pop_address(memory);
                if let Some(address) = popped {
                    processor.ip = address;
                }
            }
            Instruction::JMPIF => {
                let condition = processor.pop();
                let popped = processor.pop_address(memory);
                if condition == 0 {
                    return;
                }
                if let Some(address) = popped {
                    processor.ip = address;
                }
            }
            Instruction::CALL => {
                let popped = processor.pop_address(memory);
                processor.push(processor.ip as u64);
                if let Some(address) = popped {
                    processor.ip = address;
                }
            }
            Instruction::CALLIF => {
                let condition = processor.pop();
                let popped = processor.pop_address(memory);
                if condition == 0 {
                    return;
                }
                processor.push(processor.ip as u64);
                if let Some(address) = popped {
                    processor.ip = address;
                }
            }

            // Memory
            Instruction::ADDR => {
                processor.push(processor.ip as u64);
            }
            Instruction::READ => {
                let popped = processor.pop_address(memory);
                let value = match popped {
                    Some(address) => memory.values[address],
                    // out of bounds address
                    None => u8::MAX,
                };
                processor.push(value as u64);
            }
            Instruction::WRITE => {
                let value = processor.pop();
                let popped = processor.pop_address(memory);
                match popped {
                    Some(address) => {
                        let constrained_value = if value >= u8::MAX as u64 {
                            u8::MAX
                        } else {
                            // truncate
                            value as u8
                        };
                        memory.values[address] = constrained_value;
                    }
                    None => {
                        // no write out of bounds
                    }
                }
            }

            _ => panic!("unsupported instruction"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;
    use crate::assembler::Assembler;

    #[test]
    fn test_decode_success() {
        assert_eq!(Instruction::decode(0), Some(Instruction::NOOP));
    }

    #[test]
    fn test_decode_failure() {
        assert_eq!(Instruction::decode(u8::MAX), None);
    }

    #[test]
    fn test_add() {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        assembler.assemble("N1 N2 ADD", &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        processor.execute_amount(&mut memory, &mut small_rng, 3);

        assert_eq!(processor.current_stack(), [3]);
    }

    #[test]
    fn test_sub() {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        assembler.assemble("N4 N2 SUB", &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        processor.execute_amount(&mut memory, &mut small_rng, 3);

        assert_eq!(processor.current_stack(), [2]);
    }

    #[test]
    fn test_mul() {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        assembler.assemble("N4 N2 MUL", &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        processor.execute_amount(&mut memory, &mut small_rng, 3);

        assert_eq!(processor.current_stack(), [8]);
    }

    #[test]
    fn test_div() {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        assembler.assemble("N8 N2 DIV", &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        processor.execute_amount(&mut memory, &mut small_rng, 3);

        assert_eq!(processor.current_stack(), [4]);
    }

    #[test]
    fn test_div_by_zero() {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        assembler.assemble("N8 N2 N2 SUB DIV", &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        processor.execute_amount(&mut memory, &mut small_rng, 5);

        assert_eq!(processor.current_stack(), [0]);
    }

    #[test]
    fn test_mod() {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        assembler.assemble("N8 N2 N1 ADD MOD", &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        processor.execute_amount(&mut memory, &mut small_rng, 5);

        assert_eq!(processor.current_stack(), [2]);
    }

    #[test]
    fn test_mod_by_zero() {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        assembler.assemble("N8 N2 N2 SUB MOD", &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        processor.execute_amount(&mut memory, &mut small_rng, 5);

        assert_eq!(processor.current_stack(), [0]);
    }

    #[test]
    fn test_rnd() {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        assembler.assemble("RND RND", &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);

        processor.execute(&mut memory, &mut small_rng);
        processor.execute(&mut memory, &mut small_rng);

        assert_eq!(
            processor.current_stack(),
            [5987356902031041503, 7051070477665621255]
        );
    }
}
