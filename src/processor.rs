use moveslice::Moveslice;
use rand::rngs::SmallRng;

use crate::direction::Direction;
use crate::instruction::Instruction;
use crate::memory::Memory;
use serde_big_array::BigArray;
use serde_derive::{Deserialize, Serialize};

const STACK_SIZE: usize = 64;
pub const HEADS_AMOUNT: usize = 10;
const MAX_ADDRESS_DISTANCE: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Processor {
    pub ip: usize,
    stack_pointer: usize,
    jumped: bool,
    pub alive: bool,
    pub want_start: Option<usize>,
    pub want_split: Option<(Direction, usize)>,
    pub want_merge: Option<Direction>,
    pub want_eat: bool,
    pub want_grow: bool,
    pub current_head: usize,
    heads: [Option<usize>; HEADS_AMOUNT],
    #[serde(with = "BigArray")]
    stack: [u64; STACK_SIZE],
}

impl Processor {
    pub fn new(ip: usize) -> Processor {
        Processor {
            ip,
            current_head: 0,
            heads: [None; HEADS_AMOUNT],
            stack: [0; STACK_SIZE],
            jumped: false,
            alive: true,
            want_start: None,
            want_split: None,
            want_merge: None,
            want_eat: false,
            want_grow: false,
            stack_pointer: 0,
        }
    }

    pub fn current_stack(&self) -> &[u64] {
        &self.stack[0..self.stack_pointer]
    }

    pub fn execute(&mut self, memory: &mut Memory, rng: &mut SmallRng) -> bool {
        if !self.alive {
            return false;
        }
        if self.ip >= memory.values.len() {
            self.alive = false;
            return false;
        }
        let value = memory.values[self.ip];
        if let Some(instruction) = Instruction::decode(value) {
            instruction.execute(self, memory, rng);
        } // any other instruction is a noop
        if !self.jumped {
            self.ip += 1;
        } else {
            self.jumped = false;
        }
        true
    }

    pub fn execute_amount(
        &mut self,
        memory: &mut Memory,
        rng: &mut SmallRng,
        amount: usize,
    ) -> usize {
        self.want_start = None;
        self.want_eat = false;
        self.want_grow = false;
        self.want_split = None;
        self.want_merge = None;
        let mut total = 0;
        for _ in 0..amount {
            if self.execute(memory, rng) {
                total += 1;
            }
        }
        total
    }

    pub fn start(&mut self, address: usize) {
        self.want_start = Some(address);
    }

    pub fn end(&mut self) {
        self.alive = false;
    }

    pub fn jump(&mut self, address: usize) {
        self.ip = address;
        self.jumped = true;
    }

    pub fn call(&mut self, address: usize) {
        self.push(self.ip as u64 + 1);
        self.jump(address);
    }

    pub fn set_current_head(&mut self, value: usize) {
        self.heads[self.current_head] = Some(value);
    }

    pub fn get_current_head(&self) -> Option<usize> {
        self.heads[self.current_head]
    }

    pub fn pop_head_nr(&mut self) -> usize {
        let value = self.pop_clamped(HEADS_AMOUNT as u64);
        value as usize
    }

    pub fn get_head(&self, head_nr: usize) -> Option<usize> {
        self.heads[head_nr]
    }

    pub fn forward_current_head(&mut self, amount: usize, memory: &Memory) {
        if let Some(value) = self.heads[self.current_head] {
            let new_value = value + amount;
            if new_value >= memory.values.len() {
                return;
            }
            if self.address_distance(new_value) > MAX_ADDRESS_DISTANCE {
                return;
            }
            self.heads[self.current_head] = Some(new_value);
        }
    }

    pub fn backward_current_head(&mut self, amount: usize) {
        if let Some(value) = self.heads[self.current_head] {
            if amount > value {
                return;
            }
            let new_value = value - amount;
            if self.address_distance(new_value) > MAX_ADDRESS_DISTANCE {
                return;
            }
            self.heads[self.current_head] = Some(new_value);
        }
    }

    fn address_distance(&self, address: usize) -> usize {
        if address > self.ip {
            address - self.ip
        } else {
            self.ip - address
        }
    }

    pub fn address(&self) -> u64 {
        self.ip as u64
    }

    pub fn push(&mut self, value: u64) {
        if self.stack_pointer >= STACK_SIZE {
            self.compact_stack();
        }
        self.stack[self.stack_pointer] = value;
        self.stack_pointer += 1;
    }

    fn compact_stack(&mut self) {
        self.stack_pointer = STACK_SIZE / 2;
        self.stack.moveslice(self.stack_pointer.., 0);
    }

    pub fn dup(&mut self) {
        if self.stack_pointer < 1 {
            return;
        }
        self.push(self.top());
    }

    pub fn dup2(&mut self) {
        if self.stack_pointer < 2 {
            return;
        }
        let first = self.stack[self.stack_pointer - 2];
        let second = self.stack[self.stack_pointer - 1];
        self.push(first);
        self.push(second);
    }

    pub fn pop(&mut self) -> u64 {
        if self.stack_pointer == 0 {
            return u64::MAX;
        }
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer]
    }

    pub fn pop_clamped(&mut self, amount: u64) -> u64 {
        let value = self.pop();
        value % amount
    }

    pub fn pop_address(&mut self, memory: &Memory) -> Option<usize> {
        if self.stack_pointer == 0 {
            return None;
        }
        self.stack_pointer -= 1;
        let result = self.stack[self.stack_pointer] as usize;
        if result >= memory.values.len() {
            return None;
        }
        let distance = if result > self.ip {
            result - self.ip
        } else {
            self.ip - result
        };
        if distance > MAX_ADDRESS_DISTANCE {
            return None;
        }
        Some(result)
    }

    fn top(&self) -> u64 {
        self.stack[self.stack_pointer - 1]
    }

    pub fn drop_top(&mut self) {
        if self.stack_pointer == 0 {
            return;
        }
        self.stack_pointer -= 1;
    }

    pub fn swap(&mut self) {
        if self.stack_pointer < 2 {
            return;
        }
        let under = self.stack_pointer - 2;
        let over = self.stack_pointer - 1;
        self.stack.swap(over, under);
    }

    pub fn over(&mut self) {
        if self.stack_pointer < 2 {
            return;
        }
        let under = self.stack_pointer - 2;
        self.push(self.stack[under]);
    }

    pub fn rot(&mut self) {
        if self.stack_pointer < 3 {
            return;
        }
        let one = self.stack_pointer - 3;
        let two = self.stack_pointer - 2;
        let three = self.stack_pointer - 1;
        let temp = self.stack[one];
        self.stack[one] = self.stack[two];
        self.stack[two] = self.stack[three];
        self.stack[three] = temp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_stack() {
        let mut processor = Processor::new(0);
        let stack_size: u64 = STACK_SIZE.try_into().unwrap();
        for value in 0..stack_size {
            processor.push(value);
        }
        assert_eq!(processor.stack_pointer, STACK_SIZE);
        assert_eq!(processor.top(), stack_size - 1);

        // push one more item which should cause stack compaction
        processor.push(100);

        assert_eq!(processor.stack_pointer, STACK_SIZE / 2 + 1);
        assert_eq!(processor.top(), 100);
    }

    #[test]
    fn test_pop() {
        let mut processor = Processor::new(0);
        processor.push(10);
        processor.push(100);
        assert_eq!(processor.pop(), 100);
        assert_eq!(processor.pop(), 10);
    }

    #[test]
    fn test_pop_empty_stack() {
        let mut processor = Processor::new(0);
        processor.push(10);
        assert_eq!(processor.pop(), 10);
        assert_eq!(processor.pop(), u64::MAX);
    }

    #[test]
    fn test_pop_clamped() {
        let mut processor = Processor::new(0);
        processor.push(5);
        assert_eq!(processor.pop_clamped(6), 5);
        processor.push(10);
        assert_eq!(processor.pop_clamped(6), 4);
        assert_eq!(processor.pop_clamped(6), 3);
    }

    #[test]
    fn test_pop_head_nr() {
        let mut processor = Processor::new(0);
        processor.push(5);
        assert_eq!(processor.pop_head_nr(), 5);
        processor.push(10);
        assert_eq!(processor.pop_head_nr(), 0);
        assert_eq!(processor.pop_head_nr(), 5);
    }

    #[test]
    fn test_get_current_head_not_yet_set() {
        let processor = Processor::new(0);
        assert_eq!(processor.get_current_head(), None);
    }

    #[test]
    fn test_get_current_head_after_set() {
        let mut processor = Processor::new(0);
        processor.set_current_head(10);
        assert_eq!(processor.get_current_head(), Some(10));
    }

    #[test]
    fn test_forward_current_head() {
        let memory = Memory::new(100);
        let mut processor = Processor::new(0);
        processor.set_current_head(10);
        processor.forward_current_head(14, &memory);
        assert_eq!(processor.get_current_head(), Some(24));
    }

    #[test]
    fn test_forward_current_head_out_of_bounds_memory() {
        let memory = Memory::new(100);
        let mut processor = Processor::new(0);
        processor.set_current_head(10);
        processor.forward_current_head(100, &memory);
        assert_eq!(processor.get_current_head(), Some(10));
    }

    #[test]
    fn test_forward_current_head_out_of_bounds_address_distance() {
        let memory = Memory::new(MAX_ADDRESS_DISTANCE * 2);
        let mut processor = Processor::new(0);
        processor.set_current_head(10);
        processor.forward_current_head(MAX_ADDRESS_DISTANCE + 1, &memory);
        assert_eq!(processor.get_current_head(), Some(10));
    }

    #[test]
    fn test_backward_current_head() {
        let mut processor = Processor::new(0);
        processor.set_current_head(50);
        processor.backward_current_head(10);
        assert_eq!(processor.get_current_head(), Some(40));
    }

    #[test]
    fn test_backward_current_head_out_of_bounds_address_distance() {
        let mut processor = Processor::new(MAX_ADDRESS_DISTANCE * 2);
        processor.set_current_head(MAX_ADDRESS_DISTANCE + 10);
        processor.backward_current_head(MAX_ADDRESS_DISTANCE + 1);
        assert_eq!(
            processor.get_current_head(),
            Some(MAX_ADDRESS_DISTANCE + 10)
        );
    }

    #[test]
    fn test_pop_address() {
        let memory = Memory::new(100);
        let mut processor = Processor::new(0);
        processor.push(10);
        assert_eq!(processor.pop_address(&memory), Some(10));
        assert_eq!(processor.pop_address(&memory), None);
    }

    #[test]
    fn test_pop_address_out_of_bounds_of_memory() {
        let memory = Memory::new(100);
        let mut processor = Processor::new(0);
        processor.push(1000);
        assert_eq!(processor.pop_address(&memory), None);
    }

    #[test]
    fn test_pop_address_beyond_address_distance() {
        let memory = Memory::new(MAX_ADDRESS_DISTANCE * 10);
        let mut processor = Processor::new(0);
        let address_distance: u64 = MAX_ADDRESS_DISTANCE.try_into().unwrap();
        processor.push(address_distance + 1); // cannot address this
        assert_eq!(processor.pop_address(&memory), None);
    }

    #[test]
    fn test_pop_address_beyond_address_distance_other_direction() {
        let memory = Memory::new(MAX_ADDRESS_DISTANCE * 10);
        let mut processor = Processor::new(MAX_ADDRESS_DISTANCE * 2);
        processor.push(0); // cannot address this
        assert_eq!(processor.pop_address(&memory), None);
    }

    #[test]
    fn test_drop() {
        let mut processor = Processor::new(0);
        processor.push(10);
        processor.push(100);
        processor.drop_top();
        assert_eq!(processor.pop(), 10);
    }

    #[test]
    fn test_swap() {
        let mut processor = Processor::new(0);
        processor.push(1);
        processor.push(2);
        processor.swap();
        assert_eq!(processor.pop(), 1);
        assert_eq!(processor.pop(), 2);
    }

    #[test]
    fn test_swap_not_enough_on_stack() {
        let mut processor = Processor::new(0);
        processor.push(1);
        processor.swap();
        assert_eq!(processor.pop(), 1);
    }

    #[test]
    fn test_over() {
        let mut processor = Processor::new(0);
        processor.push(1);
        processor.push(2);
        processor.over();
        assert_eq!(processor.pop(), 1);
        assert_eq!(processor.pop(), 2);
        assert_eq!(processor.pop(), 1);
    }

    #[test]
    fn test_over_not_enough_on_stack() {
        let mut processor = Processor::new(0);
        processor.push(1);
        processor.over();
        assert_eq!(processor.pop(), 1);
        assert_eq!(processor.pop(), u64::MAX);
    }

    #[test]
    fn test_rot() {
        let mut processor = Processor::new(0);
        processor.push(1);
        processor.push(2);
        processor.push(3);
        processor.rot();
        assert_eq!(processor.pop(), 1);
        assert_eq!(processor.pop(), 3);
        assert_eq!(processor.pop(), 2);
    }
}
