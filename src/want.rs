use crate::direction::Direction;
use serde_big_array::BigArray;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum Want {
    Start,
    Shrink,
    Grow,
    Eat,
    Split,
    Merge,
}

const WANT_SIZE: usize = 64;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum WantArg {
    Address(usize),
    Amount(u64),
    Direction(Direction),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum WantItem {
    Want(Want, WantArg),
    Cancel(Want),
    // Block(Want, Direction)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Wants {
    #[serde(with = "BigArray")]
    want_items: [WantItem; WANT_SIZE],
    pointer: usize,
}

struct CombinedWants<'a> {
    wants: &'a [&'a Wants],
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

pub fn want_split(address: usize) -> WantItem {
    WantItem::Want(Want::Split, WantArg::Address(address))
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

    fn want_count(&self, want_item: WantItem) -> i32 {
        let mut count = 0;
        for i in 0..self.pointer {
            if want_item == self.want_items[i] {
                count += 1;
            }
        }
        count
    }

    fn cancel_count(&self, want: Want) -> i32 {
        let mut count = 0;
        for i in 0..self.pointer {
            if self.want_items[i] == WantItem::Cancel(want) {
                count += 1;
            }
        }
        count
    }

    pub fn combine<'a>(want_items: &'a [&'a Wants]) -> CombinedWants<'a> {
        CombinedWants { wants: want_items }
    }
}

impl<'a> CombinedWants<'a> {
    pub fn want_count(&self, want_item: WantItem) -> i32 {
        match want_item {
            WantItem::Want(want, arg) => {
                let mut total: i32 = 0;
                for wants in self.wants {
                    let count = wants.want_count(WantItem::Want(want, arg));
                    if count > 0 {
                        total += count;
                    } else {
                        // if we don't want it, we may want to cancel it
                        let cancel_count = wants.cancel_count(want);
                        total -= cancel_count;
                    }
                }
                total
            }
            WantItem::Cancel(want) => {
                panic!("Wannot count cancels")
            }
        }
    }
}

// #[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
// enum Want {
//     // affect computer
//     Start(usize),
//     Shrink(u64),
//     Grow(u64),
//     // affect location
//     Eat(u64),

//     // affect other computers
//     Split(usize),
//     Merge(Direction),
// }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_want_start() {
        let mut wants = Wants::new();
        wants.add(want_start(0));

        assert_eq!(wants.want_count(want_start(0)), 1);
        assert_eq!(wants.want_count(want_start(1)), 0);
        assert_eq!(wants.want_count(want_split(1)), 0);
    }

    #[test]
    fn test_want_with_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants1.add(WantItem::Cancel(Want::Start));

        let a = [&wants0, &wants1];
        let combined = Wants::combine(a.as_slice());

        assert_eq!(combined.want_count(want_start(0)), 0);
    }

    #[test]
    fn test_want_with_different_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants1.add(WantItem::Cancel(Want::Split));

        let a = [&wants0, &wants1];
        let combined = Wants::combine(a.as_slice());

        assert_eq!(combined.want_count(want_start(0)), 1);
    }

    #[test]
    fn test_want_with_cancel_from_self() {
        let mut wants0 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(WantItem::Cancel(Want::Start));

        let a = [&wants0];
        let combined = Wants::combine(a.as_slice());

        // cancel from self has no effect
        assert_eq!(combined.want_count(want_start(0)), 1);
    }

    #[test]
    fn test_want_with_cancel_from_self_different_want() {
        let mut wants0 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(WantItem::Cancel(Want::Split));

        let a = [&wants0];
        let combined = Wants::combine(a.as_slice());

        // cancel from self has no effect
        assert_eq!(combined.want_count(want_start(0)), 1);
    }

    #[test]
    fn test_really_want_start() {
        let mut wants = Wants::new();
        wants.add(want_start(0));
        wants.add(want_start(0));

        assert_eq!(wants.want_count(want_start(0)), 2);
    }

    #[test]
    fn test_really_want_one_cancel() {
        let mut wants0 = Wants::new();
        let mut wants1 = Wants::new();

        wants0.add(want_start(0));
        wants0.add(want_start(0));

        wants1.add(WantItem::Cancel(Want::Start));

        let a = [&wants0, &wants1];
        let combined = Wants::combine(a.as_slice());

        assert_eq!(combined.want_count(want_start(0)), 1);
    }
    #[test]
    fn test_want_start_too_many() {
        let mut wants = Wants::new();
        for i in 0..(WANT_SIZE + 1) {
            wants.add(want_start(0));
        }

        assert_eq!(wants.want_count(want_start(0)), WANT_SIZE as i32);
    }

    #[test]
    fn test_clear() {
        let mut wants = Wants::new();
        wants.add(want_start(0));
        wants.add(want_split(0));

        assert_eq!(wants.want_count(want_start(0)), 1);
        assert_eq!(wants.want_count(want_split(0)), 1);

        wants.clear();

        assert_eq!(wants.want_count(want_start(0)), 0);
        assert_eq!(wants.want_count(want_split(0)), 0);
    }
}
