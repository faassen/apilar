use rustc_hash::FxHashMap;

use crate::direction::Direction;
use rand::rngs::SmallRng;
use rand::Rng;
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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Hash, Eq, PartialEq, Deserialize, Serialize)]
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

pub struct WantsResult(FxHashMap<Want, FxHashMap<WantItem, i32>>);

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

    pub fn add_cancel(&mut self, want: Want) {
        self.add(WantItem::Cancel(want));
    }

    pub fn clear(&mut self) {
        self.pointer = 0;
    }

    pub fn counts_cancels(&self) -> (WantsResult, FxHashMap<Want, i32>) {
        let mut counts = FxHashMap::default();
        let mut cancels = FxHashMap::default();

        for i in 0..self.pointer {
            let want_item = self.want_items[i];
            match want_item {
                WantItem::Want(want, _arg) => {
                    *counts
                        .entry(want)
                        .or_insert_with(FxHashMap::default)
                        .entry(want_item)
                        .or_insert(0) += 1;
                }
                WantItem::Cancel(want) => *cancels.entry(want).or_insert(0) += 1,
                WantItem::Invalid => {
                    // no op
                }
            }
        }
        (WantsResult(counts), cancels)
    }

    pub fn combine<'a, T>(combined_wants: T) -> WantsResult
    where
        T: IntoIterator<Item = &'a Wants>,
    {
        let mut result = FxHashMap::default();
        for wants in combined_wants {
            let (WantsResult(counts), cancels) = wants.counts_cancels();
            for (want, want_items) in counts {
                for (want_item, count) in want_items {
                    if let WantItem::Want(_want, _arg) = want_item {
                        *result
                            .entry(want)
                            .or_insert_with(FxHashMap::default)
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
        WantsResult(result)
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

impl WantsResult {
    fn get_all(&self, want: Want) -> Vec<&WantArg> {
        self.0
            .get(&want)
            .map(|want_items| {
                want_items
                    .iter()
                    .filter(|(_, count)| *count > &0)
                    .map(|(want_item, _count)| match want_item {
                        WantItem::Want(_, arg) => arg,
                        _ => panic!("WantsResult::get_all: want_item is not Want"),
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    fn get_one(&self, want: Want, rng: &mut SmallRng) -> Option<&WantArg> {
        let want_args = self.get_all(want);
        if want_args.is_empty() {
            return None;
        }
        Some(want_args[rng.gen_range(0..want_args.len())])
    }

    // a faster implementation
    // go through all the wants and count specifically,

    pub fn want_start(&self) -> Vec<usize> {
        self.get_all(Want::Start)
            .iter()
            .map(|want_item| match want_item {
                WantArg::Address(address) => *address,
                _ => panic!("want_start called on non-start want"),
            })
            .collect()
    }

    pub fn want_shrink(&self, rng: &mut SmallRng) -> Option<u64> {
        self.get_one(Want::Shrink, rng)
            .map(|want_item| match want_item {
                WantArg::Amount(amount) => *amount,
                _ => panic!("want_shrink called on non-shrink want"),
            })
    }

    pub fn want_grow(&self, rng: &mut SmallRng) -> Option<u64> {
        self.get_one(Want::Grow, rng)
            .map(|want_item| match want_item {
                WantArg::Amount(amount) => *amount,
                _ => panic!("want_grow called on non-grow want"),
            })
    }

    pub fn want_eat(&self, rng: &mut SmallRng) -> Option<u64> {
        self.get_one(Want::Eat, rng)
            .map(|want_item| match want_item {
                WantArg::Amount(amount) => *amount,
                _ => panic!("want_eat called on non-eat want"),
            })
    }

    pub fn want_split(&self, rng: &mut SmallRng) -> Option<(Direction, usize)> {
        self.get_one(Want::Split, rng)
            .map(|want_item| match want_item {
                WantArg::DirectionAddress(direction, address) => (*direction, *address),
                _ => panic!("want_split called on non-split want"),
            })
    }

    pub fn want_merge(&self, rng: &mut SmallRng) -> Option<Direction> {
        self.get_one(Want::Merge, rng)
            .map(|want_item| match want_item {
                WantArg::Direction(direction) => *direction,
                _ => panic!("want_merge called on non-merge want"),
            })
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
    use rand::SeedableRng;

    #[test]
    fn test_want_start() {
        let mut wants = Wants::new();
        wants.add(want_start(0));

        let result = Wants::combine([&wants]);

        assert_eq!(result.want_start(), vec![0]);
        assert!(result.get_all(Want::Split).is_empty());
    }

    #[test]
    fn test_want_start_multiple() {
        let mut wants = Wants::new();
        wants.add(want_start(0));
        wants.add(want_start(10));

        let result = Wants::combine(&[wants]);

        let mut want_items = result.want_start();
        want_items.sort();
        assert_eq!(want_items, vec![0, 10]);
    }

    #[test]
    fn test_want_split_multiple() {
        let mut wants = Wants::new();
        wants.add(want_split(Direction::North, 0));
        wants.add(want_split(Direction::South, 10));

        let result = Wants::combine(&[wants]);

        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(result.want_split(&mut rng), Some((Direction::South, 10)));
    }

    #[test]
    fn test_want_with_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants1.add(WantItem::Cancel(Want::Start));

        let result = Wants::combine([&wants0, &wants1]);

        assert!(result.want_start().is_empty());
    }

    #[test]
    fn test_want_with_different_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants1.add(WantItem::Cancel(Want::Split));

        let result = Wants::combine([&wants0, &wants1]);

        assert_eq!(result.want_start(), vec![0]);
    }

    #[test]
    fn test_want_with_cancel_from_self() {
        let mut wants0 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(WantItem::Cancel(Want::Start));

        let result = Wants::combine([&wants0]);

        assert!(result.want_start().is_empty());
    }

    #[test]
    fn test_want_with_cancel_from_self_different_want() {
        let mut wants0 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(WantItem::Cancel(Want::Split));

        let result = Wants::combine([&wants0]);

        assert_eq!(result.want_start(), vec![0]);
    }

    #[test]
    fn test_really_want_start() {
        let mut wants = Wants::new();
        wants.add(want_start(0));
        wants.add(want_start(0));

        let result = Wants::combine([&wants]);

        assert_eq!(result.want_start(), vec![0]);
    }

    #[test]
    fn test_really_want_one_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(want_start(0));

        wants1.add(WantItem::Cancel(Want::Start));

        let result = Wants::combine([&wants0, &wants1]);

        assert_eq!(result.want_start(), vec![0]);
    }
    #[test]
    fn test_want_start_too_many() {
        let mut wants = Wants::new();
        wants.add(want_start(10));
        for _ in 0..WANT_SIZE - 1 {
            wants.add(want_start(0));
        }
        // this is not accepted
        wants.add(want_start(20));

        let result = Wants::combine([&wants]);

        let mut items = result.want_start();
        items.sort();

        assert_eq!(items, vec![0, 10]);
    }

    #[test]
    fn test_clear() {
        let mut wants = Wants::new();
        wants.add(want_start(0));
        wants.add(want_split(Direction::North, 0));

        let mut rng = SmallRng::from_seed([0; 32]);

        {
            let result = Wants::combine([&wants]);

            assert_eq!(result.want_start(), vec![0]);
            assert_eq!(result.want_split(&mut rng), Some((Direction::North, 0)));
        }
        wants.clear();

        let result = Wants::combine([&wants]);

        assert!(result.want_start().is_empty());
        assert_eq!(result.want_split(&mut rng), None);
    }

    #[test]
    fn test_address_forward() {
        let mut wants = Wants::new();

        wants.add(want_start(10));
        wants.add(want_split(Direction::North, 20));

        wants.address_forward(0, 10);

        let result = Wants::combine([&wants]);
        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(result.want_start(), vec![20]);
        assert_eq!(result.want_split(&mut rng), Some((Direction::North, 30)));
    }

    #[test]
    fn test_address_backward() {
        let mut wants = Wants::new();

        wants.add(want_start(10));
        wants.add(want_split(Direction::North, 20));

        wants.address_backward(0, 5);

        let result = Wants::combine([&wants]);
        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(result.want_start(), vec![5]);
        assert_eq!(result.want_split(&mut rng), Some((Direction::North, 15)));
    }
}
