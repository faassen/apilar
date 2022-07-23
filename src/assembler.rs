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

    fn assemble_words(&self, words: Vec<&str>, memory: &mut Memory, index: usize) -> usize {
        let mut i = index;
        for word in words {
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

    pub fn assemble(&self, text: &str, memory: &mut Memory, index: usize) -> usize {
        return self.assemble_words(text.split_whitespace().collect(), memory, index);
    }

    pub fn line_assemble(&self, text: &str, memory: &mut Memory, index: usize) -> usize {
        let words = text
            .split("\n")
            .map(|line| line.split("#").collect::<Vec<&str>>()[0].trim())
            .filter(|line| !line.is_empty())
            .collect();
        return self.assemble_words(words, memory, index);
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

    #[test]
    fn test_line_assemble() {
        let mut memory = Memory::new(10);
        let assembler = Assembler::new();
        let amount = assembler.line_assemble(
            "
        N1 # 1

        # explanatory comment
        N2 # 2",
            &mut memory,
            0,
        );
        assert_eq!(amount, 2);
        assert_eq!(memory.values[0..2], [1, 2]);
    }
}
