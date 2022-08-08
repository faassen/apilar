use crate::direction::Direction;
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use serde_derive::{Deserialize, Serialize};

const MAX_ARGS: usize = 16;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Counts<T: Eq> {
    wants: [(i32, T); MAX_ARGS],
    pointer: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Wants {
    pub start: Counts<usize>,
    pub shrink: Counts<u64>,
    pub grow: Counts<u64>,
    pub eat: Counts<u64>,
    pub split: Counts<(Direction, usize)>,
    pub merge: Counts<Direction>,
}

impl<T: Eq + Copy + Default> Counts<T> {
    fn new() -> Counts<T> {
        Counts {
            wants: [(0, T::default()); MAX_ARGS],
            pointer: 0,
        }
    }

    pub fn want(&mut self, value: T) {
        for i in 0..self.pointer {
            if self.wants[i].1 == value {
                self.wants[i].0 += 1;
                return;
            }
        }
        if self.pointer >= MAX_ARGS {
            return;
        }
        self.wants[self.pointer].0 += 1;
        self.wants[self.pointer].1 = value;
        self.pointer += 1;
    }

    pub fn cancel(&mut self) {
        for i in 0..self.pointer {
            self.wants[i].0 -= 1;
        }
    }

    pub fn clear(&mut self) {
        self.pointer = 0;
    }

    pub fn get(&self) -> impl Iterator<Item = T> + '_ {
        self.wants[0..self.pointer].iter().filter_map(
            |(count, value)| {
                if *count > 0 {
                    Some(*value)
                } else {
                    None
                }
            },
        )
    }

    pub fn choose(&self, rng: &mut SmallRng) -> Option<T> {
        self.get().choose(rng)
    }
}

impl Wants {
    pub fn new() -> Wants {
        Wants {
            start: Counts::new(),
            shrink: Counts::new(),
            grow: Counts::new(),
            eat: Counts::new(),
            split: Counts::new(),
            merge: Counts::new(),
        }
    }

    pub fn clear(&mut self) {
        self.start.clear();
        self.shrink.clear();
        self.grow.clear();
        self.eat.clear();
        self.split.clear();
        self.merge.clear();
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::SeedableRng;

    type Result = Vec<usize>;

    #[test]
    fn test_want_start() {
        let mut wants = Wants::new();
        wants.start.want(0);

        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(wants.start.get().collect::<Result>(), vec![0]);
        assert_eq!(wants.split.choose(&mut rng), None)
    }

    #[test]
    fn test_want_start_multiple() {
        let mut wants = Wants::new();

        wants.start.want(0);
        wants.start.want(10);

        let mut results: Result = wants.start.get().collect();
        results.sort();
        assert_eq!(results, vec![0, 10]);
    }

    #[test]
    fn test_want_split_multiple() {
        let mut wants = Wants::new();
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::South, 10));

        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(wants.split.choose(&mut rng), Some((Direction::South, 10)));
    }

    #[test]
    fn test_want_with_cancel() {
        let mut wants = Wants::new();

        wants.start.want(0);
        wants.start.cancel();

        assert!(wants.start.get().next().is_none());
    }

    #[test]
    fn test_want_with_cancel_something_else() {
        let mut wants = Wants::new();

        wants.start.want(0);
        wants.start.cancel();

        assert!(wants.start.get().next().is_none());
    }

    #[test]
    fn test_really_want_start() {
        let mut wants = Wants::new();
        wants.start.want(0);
        wants.start.want(0);

        assert_eq!(wants.start.get().collect::<Result>(), vec![0]);
    }

    #[test]
    fn test_really_want_one_cancel() {
        let mut wants = Wants::new();
        wants.start.want(0);
        wants.start.want(0);
        assert_eq!(wants.start.wants[0], (2, 0));
        wants.start.cancel();
        assert_eq!(wants.start.wants[0], (1, 0));
        assert_eq!(wants.start.get().collect::<Result>(), vec![0]);
    }
    #[test]
    fn test_want_start_too_many() {
        let mut wants = Wants::new();
        wants.start.want(100);
        for i in 0..MAX_ARGS - 1 {
            wants.start.want(i);
        }
        // this is not accepted
        wants.start.want(200);

        let mut results: Result = wants.start.get().collect();
        results.sort();
        assert!(results.contains(&100));
        assert!(results.contains(&(MAX_ARGS - 2)));
        assert!(!results.contains(&200));
    }

    #[test]
    fn test_clear() {
        let mut wants = Wants::new();
        wants.start.want(0);
        wants.split.want((Direction::North, 0));

        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(wants.start.get().collect::<Result>(), vec![0]);
        assert_eq!(wants.split.choose(&mut rng), Some((Direction::North, 0)));

        wants.clear();

        assert!(wants.start.get().next().is_none());
        assert_eq!(wants.split.choose(&mut rng), None);
    }
}
