use crate::assembler::Assembler;
use crate::computer::{Sensors, SENSORS_AMOUNT};
use crate::instruction::Metabolism;
use crate::memory::Memory;
use crate::processor::Processor;
use crate::want::Wants;
use rand::rngs::SmallRng;
use rand::SeedableRng;

pub struct Exec {
    pub assembler: Assembler,
    pub processor: Processor,
    pub memory: Memory,
    pub wants: Wants,
    pub rng: SmallRng,
}

pub fn execute(text: &str) -> Exec {
    let assembler = Assembler::new();
    let mut memory = Memory::new(1000);
    let mut wants = Wants::new();
    let amount = assembler.assemble(text, &mut memory, 0);
    let mut processor = Processor::new(0);
    let sensors: Sensors = [None, Some(17), None, None, None, None, None, None];
    let mut rng = SmallRng::from_seed([0; 32]);
    processor.execute_amount(
        &mut memory,
        &sensors,
        &mut wants,
        &mut rng,
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
        wants,
        rng,
    }
}

pub fn execute_lines(text: &str) -> Exec {
    let assembler = Assembler::new();
    let mut memory = Memory::new(1000);
    let mut wants = Wants::new();
    let amount = assembler.line_assemble(text, &mut memory, 0);
    let mut processor = Processor::new(0);
    let mut rng = SmallRng::from_seed([0; 32]);
    let sensors = [None; SENSORS_AMOUNT];
    processor.execute_amount(
        &mut memory,
        &sensors,
        &mut wants,
        &mut rng,
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
        wants,
        rng,
    }
}
