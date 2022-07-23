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

fn main() {
    println!("Hello, world!");
}
