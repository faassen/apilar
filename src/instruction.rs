use crate::direction::Direction;
use crate::memory::Memory;
use crate::processor::Processor;
use rand::rngs::SmallRng;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

const MAX_MOVE_HEAD_AMOUNT: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metabolism {
    pub max_eat_amount: u64,
    pub max_grow_amount: u64,
    pub max_shrink_amount: u64,
}

#[derive(EnumIter, Debug, PartialEq, Display, FromPrimitive, ToPrimitive)]
pub enum Instruction {
    // Numbers
    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    RND, // Random number

    // stack operators
    DUP,
    DUP2,
    DROP,
    SWAP,
    OVER,
    ROT,

    // Arithmetic
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,

    // Comparison
    EQ,
    GT,
    LT,

    // Logic
    NOT,
    AND,
    OR,

    // adresssing and memory
    HEAD,
    ADDR,
    COPY,
    FORWARD,
    BACKWARD,
    DISTANCE,
    READ,
    WRITE,

    // control
    JMP,   // also serves as return
    JMPIF, // jump if boolean true,

    // PRINT0,
    // PRINT1,
    // PRINT2,

    // processors
    START, // start a new processor given a starting point (only 1 can started in execution block)
    END,   // end this processor's existence

    // split and merge
    SPLIT,
    MERGE,

    // resources
    EAT,
    GROW,
    SHRINK,
    MEMORY,

    // Noop
    NOOP = u8::MAX as isize,
}

impl Instruction {
    pub fn decode(value: u8) -> Option<Instruction> {
        num::FromPrimitive::from_u8(value)
    }

    pub fn execute(
        &self,
        processor: &mut Processor,
        memory: &mut Memory,
        rng: &mut SmallRng,
        metabolism: &Metabolism,
    ) {
        match self {
            // Instruction::PRINT0 => {
            //     println!("P0 {:?}", processor.current_stack());
            // }
            // Instruction::PRINT1 => {
            //     println!("P1 {:?}", processor.current_stack());
            // }
            // Instruction::PRINT2 => {
            //     println!("P2 {:?}", processor.current_stack());
            // }
            Instruction::NOOP => {
                // nothing
            }
            // Numbers
            Instruction::N0 => {
                processor.push(0);
            }
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
                processor.push(rng.gen::<u8>() as u64);
            }

            // Stack manipulation
            Instruction::DUP => {
                processor.dup();
            }
            Instruction::DUP2 => {
                processor.dup2();
            }
            Instruction::DROP => {
                processor.drop_top();
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

            // Heads
            Instruction::HEAD => {
                let head_nr = processor.pop_head_nr();
                processor.current_head = head_nr as usize;
            }
            Instruction::ADDR => {
                processor.set_current_head_value(processor.ip);
            }
            Instruction::COPY => {
                let head_nr = processor.pop_head_nr();

                let value = processor.get_head(head_nr);

                if let Some(value) = value {
                    processor.set_current_head_value(value)
                }
            }
            Instruction::FORWARD => {
                let amount = processor.pop() as usize;
                if amount > MAX_MOVE_HEAD_AMOUNT {
                    return;
                }
                processor.forward_current_head(amount, memory);
            }
            Instruction::BACKWARD => {
                let amount = processor.pop() as usize;
                if amount > MAX_MOVE_HEAD_AMOUNT {
                    return;
                }
                processor.backward_current_head(amount);
            }
            Instruction::DISTANCE => {
                let head_nr = processor.pop_head_nr();
                let current_address = processor.get_current_head_value();
                match current_address {
                    Some(address) => match processor.get_head(head_nr) {
                        Some(other_address) => {
                            let distance = if address > other_address {
                                address - other_address
                            } else {
                                other_address - address
                            };
                            processor.push(distance as u64);
                        }
                        None => {
                            processor.push(0);
                        }
                    },
                    None => {
                        processor.push(0);
                    }
                }
            }
            Instruction::READ => {
                let popped = processor.get_current_head_value();
                let value = match popped {
                    Some(address) => memory.values[address],
                    // out of bounds address
                    None => u8::MAX,
                };
                processor.push(value as u64);
            }
            Instruction::WRITE => {
                let value = processor.pop();
                let popped = processor.get_current_head_value();
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

            // Control
            Instruction::JMP => {
                let popped = processor.get_current_head_value();
                if let Some(address) = popped {
                    processor.jump(address);
                }
            }
            Instruction::JMPIF => {
                let condition = processor.pop();
                let popped = processor.get_current_head_value();
                if condition == 0 {
                    return;
                }
                if let Some(address) = popped {
                    processor.jump(address);
                }
            }

            // Processors
            Instruction::START => {
                let popped = processor.get_current_head_value();
                if let Some(address) = popped {
                    processor.start(address);
                }
            }

            Instruction::END => {
                processor.end();
            }

            // resources
            Instruction::EAT => {
                let amount = processor.pop_clamped(metabolism.max_eat_amount);
                processor.want_eat = Some(amount);
            }
            Instruction::GROW => {
                let amount = processor.pop_clamped(metabolism.max_grow_amount);
                processor.want_grow = Some(amount);
            }
            Instruction::SHRINK => {
                let amount = processor.pop_clamped(metabolism.max_shrink_amount);
                processor.want_shrink = Some(amount);
            }
            Instruction::MEMORY => {
                let length = memory.values.len();
                processor.push(length as u64);
            }

            // split and merge
            Instruction::SPLIT => {
                let direction = processor.pop_clamped(4);
                let popped = processor.get_current_head_value();
                if let Some(address) = popped {
                    let direction = if let Some(direction) = num::FromPrimitive::from_u64(direction)
                    {
                        direction
                    } else {
                        // XXX random instead. but shouldn't happen...
                        Direction::North
                    };
                    processor.want_split = Some((direction, address));
                }
            }

            Instruction::MERGE => {
                let direction = processor.pop();
                let direction =
                    if let Some(direction) = num::FromPrimitive::from_u8((direction % 4) as u8) {
                        direction
                    } else {
                        // XXX random instead. but shouldn't happen...
                        Direction::North
                    };
                processor.want_merge = Some(direction);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::assembler::text_to_words;
    use crate::testutil::{execute, execute_lines};

    #[test]
    fn test_decode_success() {
        assert_eq!(Instruction::decode(0), Some(Instruction::N0));
    }

    #[test]
    fn test_decode_failure() {
        assert_eq!(Instruction::decode(u8::MAX - 1), None);
    }

    #[test]
    fn test_rnd() {
        let exec = execute("RND RND");

        assert_eq!(exec.processor.current_stack(), [97, 61]);
    }

    #[test]
    fn test_add() {
        let exec = execute("N2 N1 ADD");

        assert_eq!(exec.processor.current_stack(), [3]);
    }

    #[test]
    fn test_sub() {
        let exec = execute("N4 N2 SUB");

        assert_eq!(exec.processor.current_stack(), [2]);
    }

    #[test]
    fn test_mul() {
        let exec = execute("N4 N2 MUL");

        assert_eq!(exec.processor.current_stack(), [8]);
    }

    #[test]
    fn test_div() {
        let exec = execute("N8 N2 DIV");

        assert_eq!(exec.processor.current_stack(), [4]);
    }

    #[test]
    fn test_div_by_zero() {
        let exec = execute("N8 N2 N2 SUB DIV");
        assert_eq!(exec.processor.current_stack(), [0]);
    }

    #[test]
    fn test_mod() {
        let exec = execute("N8 N2 N1 ADD MOD");
        assert_eq!(exec.processor.current_stack(), [2]);
    }

    #[test]
    fn test_mod_by_zero() {
        let exec = execute("N8 N2 N2 SUB MOD");
        assert_eq!(exec.processor.current_stack(), [0]);
    }

    #[test]
    fn test_not_positive() {
        let exec = execute("N2 NOT");
        assert_eq!(exec.processor.current_stack(), [0]);
    }

    #[test]
    fn test_not_zero() {
        let exec = execute("N2 N2 SUB NOT");
        assert_eq!(exec.processor.current_stack(), [1]);
    }

    #[test]
    fn test_eq_equal() {
        let exec = execute("N2 N2 EQ");

        assert_eq!(exec.processor.current_stack(), [1]);
    }

    #[test]
    fn test_eq_not_equal() {
        let exec = execute("N1 N2 EQ");
        assert_eq!(exec.processor.current_stack(), [0]);
    }

    #[test]
    fn test_addr() {
        let exec = execute("ADDR");
        assert_eq!(exec.processor.get_current_head_value(), Some(0));
    }

    #[test]
    fn test_addr_further() {
        let exec = execute("N1 N2 N4 ADDR");
        assert_eq!(exec.processor.get_current_head_value(), Some(3));
    }

    #[test]
    fn test_change_current_head() {
        let exec = execute("N1 HEAD ADDR");
        assert_eq!(exec.processor.get_current_head_value(), Some(2));
        assert_eq!(exec.processor.current_head, 1);
    }

    #[test]
    fn test_change_current_head_clamped() {
        let exec = execute("N7 N3 MUL HEAD ADDR");
        assert_eq!(exec.processor.current_head, 5);
        assert_eq!(exec.processor.get_current_head_value(), Some(4));
    }

    #[test]
    fn test_copy_head() {
        let exec = execute("N1 HEAD ADDR N0 HEAD N1 COPY");
        assert_eq!(exec.processor.get_current_head_value(), Some(2));
        assert_eq!(exec.processor.get_head(1), Some(2));
        assert_eq!(exec.processor.current_head, 0);
    }

    #[test]
    fn test_forward() {
        let exec = execute("N0 HEAD ADDR N2 FORWARD");
        assert_eq!(exec.processor.get_current_head_value(), Some(4));
        assert_eq!(exec.processor.current_stack(), &[] as &[u64])
    }

    #[test]
    fn test_forward_out_of_bounds() {
        let exec = execute("N0 HEAD ADDR N8 N8 MUL N8 MUL N8 MUL FORWARD");
        assert_eq!(exec.processor.get_current_head_value(), Some(2));
    }

    #[test]
    fn test_backward() {
        let exec = execute("N0 HEAD ADDR N1 BACKWARD");
        assert_eq!(exec.processor.get_current_head_value(), Some(1));
    }

    #[test]
    fn test_backward_out_of_bounds() {
        let exec = execute("N0 HEAD ADDR N3 BACKWARD");
        assert_eq!(exec.processor.get_current_head_value(), Some(2));
    }

    #[test]
    fn test_distance() {
        let exec = execute("N0 HEAD ADDR N1 HEAD ADDR N0 DISTANCE");
        assert_eq!(exec.processor.current_stack(), [3]);
    }

    #[test]
    fn test_distance_with_self() {
        let exec = execute("N0 HEAD ADDR N1 HEAD ADDR N1 DISTANCE");
        assert_eq!(exec.processor.current_stack(), [0]);
    }

    #[test]
    fn test_distance_with_reverse() {
        let exec = execute("N0 HEAD ADDR N1 HEAD ADDR N0 HEAD N1 DISTANCE");
        assert_eq!(exec.processor.current_stack(), [3]);
    }

    #[test]
    fn test_jmp() {
        let exec = execute("ADDR JMP");
        assert_eq!(exec.processor.current_stack(), &[] as &[u64]);
        assert_eq!(exec.processor.address(), 0);
    }

    #[test]
    fn test_jump_further() {
        let exec = execute("ADDR N6 FORWARD JMP N1 N2 N3 N4");
        assert_eq!(exec.processor.current_stack(), [3, 4]);
    }

    #[test]
    fn test_jmpif_true() {
        let exec = execute("ADDR N1 JMPIF");
        assert_eq!(exec.processor.current_stack(), &[] as &[u64]);
        assert_eq!(exec.processor.address(), 0);
    }

    #[test]
    fn test_jmpif_false() {
        let exec = execute("ADDR N0 JMPIF");
        assert_eq!(exec.processor.current_stack(), &[] as &[u64]);
        assert_eq!(exec.processor.address(), 3);
    }

    #[test]
    fn test_die_if_out_of_bounds() {
        let mut exec = execute("N1 N2");
        assert_eq!(exec.processor.current_stack(), [1, 2]);
        // execute two more
        exec.processor.execute_amount(
            &mut exec.memory,
            &mut exec.rng,
            1002,
            &Metabolism {
                max_eat_amount: 0,
                max_grow_amount: 0,
                max_shrink_amount: 0,
            },
        );
        assert_eq!(exec.processor.current_stack(), [1, 2]);
        assert!(!exec.processor.alive);
    }

    #[test]
    fn test_copy_self() {
        let text = "
            ADDR  # h0 = start
            N1
            HEAD   
            ADDR  # h1 = loop
            N2
            HEAD
            N0
            COPY  # h2 = h0
            N8
            N8
            MUL
            FORWARD # h2 forward 64
            N0
            HEAD 
            READ  # inst from position 0
            N1
            FORWARD # h0 forward 
            N2
            HEAD
            WRITE # write inst to h2
            N1
            FORWARD  # move h2 forward
            N1
            HEAD
            JMP   # jump back to h1, loop";

        let mut exec = execute_lines(text);
        let words = text_to_words(text);
        let words_amount = words.len();

        exec.processor.execute_amount(
            &mut exec.memory,
            &mut exec.rng,
            (words_amount - 1) * words_amount,
            &Metabolism {
                max_eat_amount: 0,
                max_grow_amount: 0,
                max_shrink_amount: 0,
            },
        );

        assert_eq!(
            exec.assembler
                .disassemble_to_words(&exec.memory.values[64..64 + words_amount]),
            words
        );
    }
}
