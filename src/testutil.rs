use crate::assembler::Assembler;
use crate::instruction::Metabolism;
use crate::memory::Memory;
use crate::processor::Processor;
use rand::rngs::SmallRng;
use rand::SeedableRng;

pub struct Exec {
    pub assembler: Assembler,
    pub processor: Processor,
    pub memory: Memory,
    pub small_rng: SmallRng,
}

pub fn execute(text: &str) -> Exec {
    let assembler = Assembler::new();
    let mut memory = Memory::new(1000);
    let amount = assembler.assemble(text, &mut memory, 0);
    let mut processor = Processor::new(0);
    let mut small_rng = SmallRng::from_seed([0; 32]);
    processor.execute_amount(
        &mut memory,
        &mut small_rng,
        amount,
        &Metabolism {
            max_eat_amount: 0,
            max_grow_amount: 0,
            max_shrink_amount: 0,
        },
    );
    Exec {
        assembler,
        processor,
        memory,
        small_rng,
    }
}

pub fn execute_lines(text: &str) -> Exec {
    let assembler = Assembler::new();
    let mut memory = Memory::new(1000);
    let amount = assembler.line_assemble(text, &mut memory, 0);
    let mut processor = Processor::new(0);
    let mut small_rng = SmallRng::from_seed([0; 32]);
    processor.execute_amount(
        &mut memory,
        &mut small_rng,
        amount,
        &Metabolism {
            max_eat_amount: 0,
            max_grow_amount: 0,
            max_shrink_amount: 0,
        },
    );
    Exec {
        assembler,
        processor,
        memory,
        small_rng,
    }
}
