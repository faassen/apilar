use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::instruction::Instruction;
use crate::memory::Memory;

pub struct Assembler {
    instructions: HashMap<String, Instruction>,
}

impl Assembler {
    pub fn new() -> Assembler {
        let mut instructions = HashMap::new();
        for instruction in Instruction::iter() {
            instructions.insert(instruction.to_string(), instruction);
        }
        return Assembler { instructions };
    }

    pub fn assemble(&self, text: &str, memory: &mut Memory, index: usize) -> usize {
        let mut i = index;
        for word in text.split_whitespace() {
            match self.instructions.get(word) {
                Some(instruction) => {
                    let v = num::ToPrimitive::to_u8(instruction);
                    if let Some(value) = v {
                        memory.write(i, value);
                    }
                }
                None => {
                    panic!("Unknown instruction: {}", word);
                }
            };
            i += 1;
        }
        return i - index;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble() {
        let mut memory = Memory::new(10);
        let assembler = Assembler::new();
        let amount = assembler.assemble("N1 N2", &mut memory, 0);
        assert_eq!(amount, 2);
        assert_eq!(memory.values[0..2], [1, 2]);
    }
}
