use std::collections::HashMap;

use crate::direction::Direction;
use serde_big_array::BigArray;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum Want {
    Start,
    Shrink,
    Grow,
    Eat,
    Split,
    Merge,
}

const WANT_SIZE: usize = 64;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum WantArg {
    Address(usize),
    DirectionAddress(Direction, usize),
    Amount(u64),
    Direction(Direction),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum WantItem {
    Want(Want, WantArg),
    Cancel(Want),
    Invalid, // has become invalid due to an adjust
             // Block(Want, Direction)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wants {
    #[serde(with = "BigArray")]
    want_items: [WantItem; WANT_SIZE],
    pointer: usize,
}

pub fn want_start(address: usize) -> WantItem {
    WantItem::Want(Want::Start, WantArg::Address(address))
}

pub fn want_grow(amount: u64) -> WantItem {
    WantItem::Want(Want::Grow, WantArg::Amount(amount))
}

pub fn want_shrink(amount: u64) -> WantItem {
    WantItem::Want(Want::Shrink, WantArg::Amount(amount))
}

pub fn want_eat(amount: u64) -> WantItem {
    WantItem::Want(Want::Eat, WantArg::Amount(amount))
}

pub fn want_split(direction: Direction, address: usize) -> WantItem {
    WantItem::Want(Want::Split, WantArg::DirectionAddress(direction, address))
}

pub fn want_merge(direction: Direction) -> WantItem {
    WantItem::Want(Want::Merge, WantArg::Direction(direction))
}

impl Wants {
    pub fn new() -> Wants {
        Wants {
            want_items: [WantItem::Cancel(Want::Start); WANT_SIZE],
            pointer: 0,
        }
    }

    pub fn add(&mut self, want_item: WantItem) {
        if self.pointer >= WANT_SIZE {
            return;
        }
        self.want_items[self.pointer] = want_item;
        self.pointer += 1;
    }

    pub fn clear(&mut self) {
        self.pointer = 0;
    }

    fn counts_cancels(&self) -> (HashMap<Want, HashMap<WantItem, i32>>, HashMap<Want, i32>) {
        let mut counts = HashMap::new();
        let mut cancels = HashMap::new();

        for i in 0..self.pointer {
            let want_item = self.want_items[i];
            match want_item {
                WantItem::Want(want, _arg) => {
                    *counts
                        .entry(want)
                        .or_insert_with(HashMap::new)
                        .entry(want_item)
                        .or_insert(0) += 1;
                }
                WantItem::Cancel(want) => *cancels.entry(want).or_insert(0) += 1,
                WantItem::Invalid => {
                    // no op
                }
            }
        }
        (counts, cancels)
    }

    pub fn combine(combined_wants: &[&Wants]) -> HashMap<Want, HashMap<WantItem, i32>> {
        let mut result = HashMap::new();
        for wants in combined_wants {
            let (counts, cancels) = wants.counts_cancels();
            for (want, want_items) in counts {
                for (want_item, count) in want_items {
                    if let WantItem::Want(_want, _arg) = want_item {
                        *result
                            .entry(want)
                            .or_insert_with(HashMap::new)
                            .entry(want_item)
                            .or_insert(0) += count;
                    }
                }
            }
            for (want, cancel_count) in cancels {
                let sub = result.get_mut(&want);
                if let Some(sub) = sub {
                    for count in sub.values_mut() {
                        *count -= cancel_count;
                    }
                }
            }
        }
        result
    }

    pub fn address_backward(&mut self, start: usize, distance: usize) {
        for i in 0..self.pointer {
            match self.want_items[i] {
                WantItem::Want(want, WantArg::Address(address)) => {
                    let adjusted = adjust_backward(address, start, distance);
                    match adjusted {
                        Some(address) => {
                            self.want_items[i] = WantItem::Want(want, WantArg::Address(address))
                        }
                        None => self.want_items[i] = WantItem::Invalid,
                    }
                }
                WantItem::Want(want, WantArg::DirectionAddress(direction, address)) => {
                    let adjusted = adjust_backward(address, start, distance);
                    match adjusted {
                        Some(address) => {
                            self.want_items[i] =
                                WantItem::Want(want, WantArg::DirectionAddress(direction, address))
                        }
                        None => self.want_items[i] = WantItem::Invalid,
                    }
                }
                _ => {
                    // no op
                }
            }
        }
    }

    pub fn address_forward(&mut self, start: usize, distance: usize) {
        for i in 0..self.pointer {
            match self.want_items[i] {
                WantItem::Want(want, WantArg::Address(address)) => {
                    if address >= start {
                        self.want_items[i] =
                            WantItem::Want(want, WantArg::Address(address + distance));
                    }
                }
                WantItem::Want(want, WantArg::DirectionAddress(direction, address)) => {
                    if address >= start {
                        self.want_items[i] = WantItem::Want(
                            want,
                            WantArg::DirectionAddress(direction, address + distance),
                        );
                    }
                }
                _ => {
                    // no op
                }
            }
        }
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

impl Default for Wants {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_want_start() {
        let mut wants = Wants::new();
        wants.add(want_start(0));

        let a = [&wants];

        let counts = Wants::combine(&a);

        let start_items = counts.get(&Want::Start).unwrap();

        assert_eq!(start_items.get(&want_start(0)), Some(&1));
        assert_eq!(start_items.get(&want_start(1)), None);
        assert_eq!(counts.get(&Want::Split), None);
    }

    #[test]
    fn test_want_with_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants1.add(WantItem::Cancel(Want::Start));

        let a = [&wants0, &wants1];

        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();

        assert_eq!(start_items.get(&want_start(0)), Some(&0));
    }

    #[test]
    fn test_want_with_different_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants1.add(WantItem::Cancel(Want::Split));

        let a = [&wants0, &wants1];
        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();

        assert_eq!(start_items.get(&want_start(0)), Some(&1))
    }

    #[test]
    fn test_want_with_cancel_from_self() {
        let mut wants0 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(WantItem::Cancel(Want::Start));

        let a = [&wants0];
        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();

        assert_eq!(start_items.get(&want_start(0)), Some(&0));
    }

    #[test]
    fn test_want_with_cancel_from_self_different_want() {
        let mut wants0 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(WantItem::Cancel(Want::Split));

        let a = [&wants0];
        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();

        // cancel of something else has no effect
        assert_eq!(start_items.get(&want_start(0)), Some(&1));
    }

    #[test]
    fn test_really_want_start() {
        let mut wants = Wants::new();
        wants.add(want_start(0));
        wants.add(want_start(0));

        let a = [&wants];
        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();

        assert_eq!(start_items.get(&want_start(0)), Some(&2));
    }

    #[test]
    fn test_really_want_one_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(want_start(0));

        wants1.add(WantItem::Cancel(Want::Start));

        let a = [&wants0, &wants1];
        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();

        assert_eq!(start_items.get(&want_start(0)), Some(&1));
    }
    #[test]
    fn test_want_start_too_many() {
        let mut wants = Wants::new();
        for _ in 0..(WANT_SIZE + 1) {
            wants.add(want_start(0));
        }

        let a = [&wants];
        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();

        assert_eq!(start_items.get(&want_start(0)), Some(&(WANT_SIZE as i32)));
    }

    #[test]
    fn test_clear() {
        let mut wants = Wants::new();
        wants.add(want_start(0));
        wants.add(want_split(Direction::North, 0));

        {
            let a = [&wants];
            let counts = Wants::combine(a.as_slice());

            let start_items = counts.get(&Want::Start).unwrap();
            let split_items = counts.get(&Want::Split).unwrap();

            assert_eq!(start_items.get(&want_start(0)), Some(&1));
            assert_eq!(split_items.get(&want_split(Direction::North, 0)), Some(&1));
        }
        wants.clear();

        let a = [&wants];
        let counts = Wants::combine(a.as_slice());

        assert_eq!(counts.get(&Want::Start), None);
        assert_eq!(counts.get(&Want::Split), None);
    }

    #[test]
    fn test_address_forward() {
        let mut wants = Wants::new();

        wants.add(want_start(10));
        wants.add(want_split(Direction::North, 20));

        wants.address_forward(0, 10);

        let a = [&wants];
        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();
        let split_items = counts.get(&Want::Split).unwrap();

        assert_eq!(start_items.get(&want_start(20)), Some(&1));
        assert_eq!(split_items.get(&want_split(Direction::North, 30)), Some(&1));
    }

    #[test]
    fn test_address_backward() {
        let mut wants = Wants::new();

        wants.add(want_start(10));
        wants.add(want_split(Direction::North, 20));

        wants.address_backward(0, 5);

        let a = [&wants];
        let counts = Wants::combine(a.as_slice());

        let start_items = counts.get(&Want::Start).unwrap();
        let split_items = counts.get(&Want::Split).unwrap();

        assert_eq!(start_items.get(&want_start(5)), Some(&1));
        assert_eq!(split_items.get(&want_split(Direction::North, 15)), Some(&1));
    }
}
