use crate::direction::Direction;
use crate::instruction::{Instruction, Metabolism};
use crate::memory::Memory;
use crate::want::Wants;
use moveslice::Moveslice;
use rand::rngs::SmallRng;
use serde_big_array::BigArray;
use serde_derive::{Deserialize, Serialize};

const STACK_SIZE: usize = 64;
pub const HEADS_AMOUNT: usize = 8;
const MAX_ADDRESS_DISTANCE: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Processor {
    pub ip: usize,
    stack_pointer: usize,
    jumped: bool,
    pub alive: bool,
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
            stack_pointer: 0,
        }
    }

    pub fn current_stack(&self) -> &[u64] {
        &self.stack[0..self.stack_pointer]
    }

    pub fn execute(
        &mut self,
        memory: &mut Memory,
        wants: &mut Wants,
        rng: &mut SmallRng,
        metabolism: &Metabolism,
    ) {
        if !self.alive {
            return;
        }
        if self.ip >= memory.values.len() {
            self.alive = false;
            return;
        }
        let value = memory.values[self.ip];
        if let Some(instruction) = Instruction::decode(value) {
            instruction.execute(self, memory, wants, rng, metabolism);
        } // any other instruction is a noop
        if !self.jumped {
            self.ip += 1;
        } else {
            self.jumped = false;
        }
    }

    pub fn execute_amount(
        &mut self,
        memory: &mut Memory,
        wants: &mut Wants,
        rng: &mut SmallRng,
        amount: usize,
        metabolism: &Metabolism,
    ) {
        for _ in 0..amount {
            self.execute(memory, wants, rng, metabolism);
        }
    }

    pub fn end(&mut self) {
        self.alive = false;
    }

    pub fn skip(&mut self) {
        self.ip += 2;
        self.jumped = true;
    }

    pub fn jump(&mut self, address: usize) {
        self.ip = address;
        self.jumped = true;
    }

    pub fn call(&mut self, address: usize) {
        self.push(self.ip as u64 + 1);
        self.jump(address);
    }

    pub fn set_current_head_value(&mut self, value: usize) {
        self.heads[self.current_head] = Some(value);
    }

    pub fn get_current_head_value(&self) -> Option<usize> {
        self.heads[self.current_head]
    }

    pub fn pop_head_nr(&mut self) -> usize {
        let value = self.pop_clamped(HEADS_AMOUNT as u64);
        value as usize
    }

    pub fn pop_direction(&mut self) -> Direction {
        let direction_nr = self.pop_clamped(4);
        if let Some(direction) = num::FromPrimitive::from_u8(direction_nr as u8) {
            direction
        } else {
            // XXX random instead. but shouldn't happen...
            Direction::North
        }
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

    pub fn adjust_backward(&mut self, address: usize, distance: usize) {
        if let Some(new_ip) = adjust_backward(self.ip, address, distance) {
            self.ip = new_ip
        } else {
            self.ip = 0;
            self.alive = false;
        }

        for i in 0..HEADS_AMOUNT {
            let head = self.heads[i];
            if let Some(head_address) = head {
                self.heads[i] = adjust_backward(head_address, address, distance);
            }
        }
    }

    pub fn adjust_forward(&mut self, address: usize, distance: usize) {
        if self.ip >= address {
            self.ip += distance;
        }

        for i in 0..HEADS_AMOUNT {
            let head = self.heads[i];
            if let Some(head_address) = head {
                if head_address >= address {
                    self.heads[i] = Some(head_address + distance);
                }
            }
        }
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
            return 0;
        }
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer]
    }

    pub fn pop_clamped(&mut self, amount: u64) -> u64 {
        let value = self.pop();
        value % amount
    }

    pub fn pop_max(&mut self, amount: u64) -> u64 {
        let value = self.pop();
        if value < amount {
            value
        } else {
            amount
        }
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

fn adjust_backward(address: usize, start: usize, distance: usize) -> Option<usize> {
    if address < start {
        return Some(address);
    }
    if address - start >= distance {
        Some(address - distance)
    } else {
        None
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
        assert_eq!(processor.pop(), 0);
    }

    #[test]
    fn test_pop_clamped() {
        let mut processor = Processor::new(0);
        processor.push(5);
        assert_eq!(processor.pop_clamped(6), 5);
        processor.push(10);
        assert_eq!(processor.pop_clamped(6), 4);
        assert_eq!(processor.pop_clamped(6), 0);
    }

    #[test]
    fn test_pop_max() {
        let mut processor = Processor::new(0);
        processor.push(5);
        assert_eq!(processor.pop_max(6), 5);
        processor.push(10);
        assert_eq!(processor.pop_max(6), 6);
        assert_eq!(processor.pop_max(6), 0);
    }

    #[test]
    fn test_pop_head_nr() {
        let mut processor = Processor::new(0);
        processor.push(5);
        assert_eq!(processor.pop_head_nr(), 5);
        processor.push(10);
        assert_eq!(processor.pop_head_nr(), 2);
        assert_eq!(processor.pop_head_nr(), 0);
    }

    #[test]
    fn test_get_current_head_not_yet_set() {
        let processor = Processor::new(0);
        assert_eq!(processor.get_current_head_value(), None);
    }

    #[test]
    fn test_get_current_head_after_set() {
        let mut processor = Processor::new(0);
        processor.set_current_head_value(10);
        assert_eq!(processor.get_current_head_value(), Some(10));
    }

    #[test]
    fn test_forward_current_head() {
        let memory = Memory::new(100);
        let mut processor = Processor::new(0);
        processor.set_current_head_value(10);
        processor.forward_current_head(14, &memory);
        assert_eq!(processor.get_current_head_value(), Some(24));
    }

    #[test]
    fn test_forward_current_head_out_of_bounds_memory() {
        let memory = Memory::new(100);
        let mut processor = Processor::new(0);
        processor.set_current_head_value(10);
        processor.forward_current_head(100, &memory);
        assert_eq!(processor.get_current_head_value(), Some(10));
    }

    #[test]
    fn test_forward_current_head_out_of_bounds_address_distance() {
        let memory = Memory::new(MAX_ADDRESS_DISTANCE * 2);
        let mut processor = Processor::new(0);
        processor.set_current_head_value(10);
        processor.forward_current_head(MAX_ADDRESS_DISTANCE + 1, &memory);
        assert_eq!(processor.get_current_head_value(), Some(10));
    }

    #[test]
    fn test_backward_current_head() {
        let mut processor = Processor::new(0);
        processor.set_current_head_value(50);
        processor.backward_current_head(10);
        assert_eq!(processor.get_current_head_value(), Some(40));
    }

    #[test]
    fn test_backward_current_head_out_of_bounds_address_distance() {
        let mut processor = Processor::new(MAX_ADDRESS_DISTANCE * 2);
        processor.set_current_head_value(MAX_ADDRESS_DISTANCE + 10);
        processor.backward_current_head(MAX_ADDRESS_DISTANCE + 1);
        assert_eq!(
            processor.get_current_head_value(),
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
        assert_eq!(processor.pop(), 0);
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

    #[test]
    fn test_adjust_forward() {
        let mut processor = Processor::new(0);

        processor.heads[0] = Some(15);
        processor.heads[1] = Some(25);

        processor.adjust_forward(0, 10);

        assert_eq!(processor.ip, 10);
        assert_eq!(processor.heads[0], Some(25));
        assert_eq!(processor.heads[1], Some(35));
    }

    #[test]
    fn test_adjust_forward_from_address() {
        let mut processor = Processor::new(0);

        processor.heads[0] = Some(15);
        processor.heads[1] = Some(25);
        processor.heads[2] = Some(10);

        processor.adjust_forward(15, 10);

        assert_eq!(processor.ip, 0);
        assert_eq!(processor.heads[0], Some(25));
        assert_eq!(processor.heads[1], Some(35));
        assert_eq!(processor.heads[2], Some(10));
    }

    #[test]
    fn test_adjust_backward() {
        let mut processor = Processor::new(5);

        processor.heads[0] = Some(15);
        processor.heads[1] = Some(25);

        processor.adjust_backward(0, 5);

        assert_eq!(processor.ip, 0);
        assert_eq!(processor.heads[0], Some(10));
        assert_eq!(processor.heads[1], Some(20));
    }

    #[test]
    fn test_adjust_backward_illegal_ip() {
        let mut processor = Processor::new(1);

        processor.adjust_backward(0, 2);

        assert_eq!(processor.ip, 0);
        assert!(!processor.alive);
    }

    #[test]
    fn test_adjust_backward_illegal_ip_after_deletion() {
        let mut processor = Processor::new(3);

        processor.adjust_backward(1, 3);

        assert_eq!(processor.ip, 0);
        assert!(!processor.alive);
    }

    #[test]
    fn test_adjust_backward_still_legal() {
        let mut processor = Processor::new(4);

        processor.adjust_backward(1, 3);

        assert_eq!(processor.ip, 1);
        assert!(processor.alive);
    }

    #[test]
    fn test_adjust_backward_illegal_head() {
        let mut processor = Processor::new(0);
        processor.heads[0] = Some(1);

        processor.adjust_backward(0, 2);

        assert_eq!(processor.heads[0], None);
    }

    #[test]
    fn test_adjust_backward_after_shrink() {
        let mut processor = Processor::new(5);

        processor.heads[0] = Some(15);
        processor.heads[1] = Some(25);

        processor.adjust_backward(6, 30);
        assert_eq!(processor.ip, 5);
        assert_eq!(processor.heads[0], None);
        assert_eq!(processor.heads[1], None);
    }
}
