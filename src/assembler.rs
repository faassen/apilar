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
        Assembler { instructions }
    }

    pub fn assemble_words(&self, words: Vec<&str>, memory: &mut Memory, index: usize) -> usize {
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
        i - index
    }

    pub fn assemble(&self, text: &str, memory: &mut Memory, index: usize) -> usize {
        self.assemble_words(text.split_whitespace().collect(), memory, index)
    }

    pub fn line_assemble(&self, text: &str, memory: &mut Memory, index: usize) -> usize {
        let words = text_to_words(text);
        self.assemble_words(words, memory, index)
    }

    pub fn disassemble_to_words(&self, values: &[u8]) -> Vec<String> {
        let mut words: Vec<String> = Vec::new();
        for value in values {
            let decoded = Instruction::decode(*value);
            match decoded {
                Some(instruction) => {
                    words.push(instruction.to_string());
                }
                None => {
                    words.push(format!("noop {}", value));
                }
            }
        }
        words
    }

    pub fn line_disassemble(&self, values: &[u8]) -> String {
        self.disassemble_to_words(values).join("\n")
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Assembler::new()
    }
}

pub fn text_to_words(text: &str) -> Vec<&str> {
    text.split('\n')
        .map(|line| line.split('#').collect::<Vec<&str>>()[0].trim())
        .filter(|line| !line.is_empty())
        .collect()
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
        assert_eq!(memory.values[0..amount], [1, 2]);
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
        assert_eq!(memory.values[0..amount], [1, 2]);
    }

    #[test]
    fn test_line_disassemble() {
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
        assert_eq!(
            assembler.line_disassemble(&memory.values[0..amount]),
            "N1\nN2"
        );
    }
}
