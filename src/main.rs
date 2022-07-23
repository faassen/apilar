extern crate num;
#[macro_use]
extern crate num_derive;

pub mod assembler;
pub mod computer;
pub mod instruction;
pub mod memory;
pub mod processor;

#[cfg(test)]
pub mod testutil;

use crate::assembler::{text_to_words, Assembler};
use crate::computer::Computer;
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn main() {
    let assembler = Assembler::new();

    let text = "
        ADDR  # c
        DUP   # preserve starting point
        ADDR  # c loop
        SWAP  # loop c
        DUP   # loop c c
        READ  # loop c inst
        SWAP  # loop inst c
        DUP   # loop inst c c
        N8
        N8
        MUL
        ADD   # loop inst c c+64
        ROT   # loop c c+64 inst
        WRITE # loop c
        N1
        ADD   # loop c+1
        DUP   # loop c+1 c+1
        ADDR
        N8
        N8
        N4
        ADD
        ADD   # add to get end of replicator
        ADD   # loop c+1 c+1 end
        LT    # loop c+1 b
        ROT   # c+1 b loop
        SWAP  # c+1 loop b
        JMPIF # go to loop
        DROP  # start
        DUP   # start start
        N8
        N8
        MUL
        ADD   # start newstart
        START # start
        JMP   # jump to first addr
        ";
    let words = text_to_words(text);

    let mut computer = Computer::new(1024 * 1024 * 1024, 100000);
    assembler.assemble_words(words.clone(), &mut computer.memory, 0);
    let mut small_rng = SmallRng::from_seed([0; 32]);

    computer.add_processor(0);
    loop {
        println!("Processors {}", computer.processors.len());
        computer.execute(&mut small_rng, 100);
    }
}
