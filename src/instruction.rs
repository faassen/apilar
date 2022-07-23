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
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
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
    START = 140, // start a new processor given a starting point (only 1 can started in execution block)
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
            Instruction::N3 => {
                processor.push(3);
            }
            Instruction::N4 => {
                processor.push(4);
            }
            Instruction::N5 => {
                processor.push(5);
            }
            Instruction::N6 => {
                processor.push(6);
            }
            Instruction::N7 => {
                processor.push(7);
            }
            Instruction::N8 => {
                processor.push(8);
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
                    processor.jump(address);
                }
            }
            Instruction::JMPIF => {
                let condition = processor.pop();
                let popped = processor.pop_address(memory);
                if condition == 0 {
                    return;
                }
                if let Some(address) = popped {
                    processor.jump(address);
                }
            }
            Instruction::CALL => {
                let popped = processor.pop_address(memory);
                if let Some(address) = popped {
                    processor.call(address);
                }
            }
            Instruction::CALLIF => {
                let condition = processor.pop();
                let popped = processor.pop_address(memory);
                if condition == 0 {
                    return;
                }
                if let Some(address) = popped {
                    processor.call(address);
                }
            }

            // Memory
            Instruction::ADDR => {
                processor.push(processor.address());
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

            // Processors
            Instruction::START => {
                let popped = processor.pop_address(memory);
                if let Some(address) = popped {
                    processor.start(address);
                }
            }

            Instruction::END => {
                processor.end();
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

    fn execute(text: &str) -> (Processor, Memory, SmallRng) {
        let assembler = Assembler::new();
        let mut memory = Memory::new(100);
        let amount = assembler.assemble(text, &mut memory, 0);
        let mut processor = Processor::new(0);
        let mut small_rng = SmallRng::from_seed([0; 32]);
        processor.execute_amount(&mut memory, &mut small_rng, amount);
        return (processor, memory, small_rng);
    }

    #[test]
    fn test_rnd() {
        let (processor, _, _) = execute("RND RND");

        assert_eq!(
            processor.current_stack(),
            [5987356902031041503, 7051070477665621255]
        );
    }

    #[test]
    fn test_add() {
        let (processor, _, _) = execute("N2 N1 ADD");

        assert_eq!(processor.current_stack(), [3]);
    }

    #[test]
    fn test_sub() {
        let (processor, _, _) = execute("N4 N2 SUB");

        assert_eq!(processor.current_stack(), [2]);
    }

    #[test]
    fn test_mul() {
        let (processor, _, _) = execute("N4 N2 MUL");

        assert_eq!(processor.current_stack(), [8]);
    }

    #[test]
    fn test_div() {
        let (processor, _, _) = execute("N8 N2 DIV");

        assert_eq!(processor.current_stack(), [4]);
    }

    #[test]
    fn test_div_by_zero() {
        let (processor, _, _) = execute("N8 N2 N2 SUB DIV");
        assert_eq!(processor.current_stack(), [0]);
    }

    #[test]
    fn test_mod() {
        let (processor, _, _) = execute("N8 N2 N1 ADD MOD");
        assert_eq!(processor.current_stack(), [2]);
    }

    #[test]
    fn test_mod_by_zero() {
        let (processor, _, _) = execute("N8 N2 N2 SUB MOD");
        assert_eq!(processor.current_stack(), [0]);
    }

    #[test]
    fn test_not_positive() {
        let (processor, _, _) = execute("N2 NOT");
        assert_eq!(processor.current_stack(), [0]);
    }

    #[test]
    fn test_not_zero() {
        let (processor, _, _) = execute("N2 N2 SUB NOT");
        assert_eq!(processor.current_stack(), [1]);
    }

    #[test]
    fn test_eq_equal() {
        let (processor, _, _) = execute("N2 N2 EQ");

        assert_eq!(processor.current_stack(), [1]);
    }

    #[test]
    fn test_eq_not_equal() {
        let (processor, _, _) = execute("N1 N2 EQ");
        assert_eq!(processor.current_stack(), [0]);
    }

    #[test]
    fn test_addr() {
        let (processor, _, _) = execute("ADDR");
        assert_eq!(processor.current_stack(), [0]);
    }

    #[test]
    fn test_addr_further() {
        let (processor, _, _) = execute("N1 N2 N4 ADDR");
        assert_eq!(processor.current_stack(), [1, 2, 4, 3]);
    }

    #[test]
    fn test_jmp() {
        let (processor, _, _) = execute("ADDR JMP");
        assert_eq!(processor.current_stack(), []);
        assert_eq!(processor.address(), 0);
    }

    #[test]
    fn test_jump_further() {
        let (processor, _, _) = execute("N2 JMP NOOP NOOP N1 N2");
        assert_eq!(processor.current_stack(), [1, 2]);
    }

    #[test]
    fn test_jmpif_true() {
        let (processor, _, _) = execute("ADDR N1 JMPIF");
        assert_eq!(processor.current_stack(), []);
        assert_eq!(processor.address(), 0);
    }

    #[test]
    fn test_jmpif_false() {
        let (processor, _, _) = execute("ADDR N1 N1 SUB JMPIF");
        assert_eq!(processor.current_stack(), []);
        assert_eq!(processor.address(), 5);
    }

    #[test]
    fn test_wrap_around_memory() {
        let (mut processor, mut memory, mut small_rng) = execute("N1 N2");
        assert_eq!(processor.current_stack(), [1, 2]);
        // execute two more
        processor.execute_amount(&mut memory, &mut small_rng, 102);
        assert_eq!(processor.current_stack(), [1, 2, 1, 2])
    }
}
