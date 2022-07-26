use crate::direction::Direction;
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use serde_derive::{Deserialize, Serialize};

const MAX_ARGS: usize = 8;

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct CountEntry<T: Eq> {
    count: i32,
    value: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Counts<T: Eq + Default> {
    wants: [CountEntry<T>; MAX_ARGS],
    cancel: i32,
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
    pub move_: Counts<Direction>,
    pub block_merge: Counts<Direction>,
}

impl<T: Eq + Copy + Default> Counts<T> {
    fn new() -> Counts<T> {
        Counts {
            wants: [CountEntry::default(); MAX_ARGS],
            cancel: 0,
            pointer: 0,
        }
    }

    pub fn want(&mut self, value: T) {
        for i in 0..self.pointer {
            if self.wants[i].value == value {
                self.wants[i].count += 1;
                return;
            }
        }
        if self.pointer >= MAX_ARGS {
            return;
        }
        self.wants[self.pointer].count += 1;
        self.wants[self.pointer].value = value;
        self.pointer += 1;
    }

    pub fn cancel(&mut self) {
        self.cancel += 1;
    }

    pub fn clear(&mut self) {
        self.pointer = 0;
    }

    fn get_strength(&self) -> impl Iterator<Item = (T, i32)> + '_ {
        self.wants[0..self.pointer].iter().filter_map(|entry| {
            let strength = entry.count - self.cancel;
            if strength > 0 {
                Some((entry.value, strength))
            } else {
                None
            }
        })
    }

    fn get_max(&self) -> i32 {
        self.get_strength()
            .map(|(_, strength)| strength)
            .max()
            .unwrap_or(0)
    }

    fn get_winners(&self) -> impl Iterator<Item = (T, i32)> + '_ {
        let max = self.get_max();
        self.get_strength()
            .filter(move |(_, strength)| strength == &max)
    }

    pub fn get(&self) -> impl Iterator<Item = T> + '_ {
        self.wants[0..self.pointer].iter().filter_map(|entry| {
            if (entry.count - self.cancel) > 0 {
                Some(entry.value)
            } else {
                None
            }
        })
    }

    pub fn get_strength_by_value(&self, value: T) -> Option<i32> {
        for (found_value, strength) in self.get_strength() {
            if found_value == value {
                return Some(strength);
            }
        }
        None
    }

    pub fn choose(&self, rng: &mut SmallRng) -> Option<T> {
        self.get_winners().choose(rng).map(|(value, _)| value)
    }

    pub fn choose_with_strength(&self, rng: &mut SmallRng) -> Option<(T, i32)> {
        self.get_winners().choose(rng)
    }
}

impl<T: Eq + Copy + Default> CountEntry<T> {
    pub fn new() -> CountEntry<T> {
        CountEntry {
            count: 0,
            value: T::default(),
        }
    }
}

impl<T: Eq + Copy + Default> Default for CountEntry<T> {
    fn default() -> Self {
        CountEntry::new()
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
            block_merge: Counts::new(),
            move_: Counts::new(),
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

impl Default for Wants {
    fn default() -> Self {
        Wants::new()
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
    fn test_want_split_multiple_winners() {
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
    fn test_want_with_cancel_ahead_of_time() {
        let mut wants = Wants::new();

        wants.start.cancel();
        wants.start.want(0);

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
        wants.start.cancel();
        assert_eq!(wants.start.get().collect::<Result>(), vec![0]);
    }

    #[test]
    fn test_really_want_really_cancel() {
        let mut wants = Wants::new();
        wants.start.cancel();
        wants.start.want(0);
        wants.start.want(0);
        wants.start.cancel();
        assert!(wants.start.get().next().is_none());
    }

    #[test]
    fn test_really_really_want_choose_strongest_wins() {
        let mut wants = Wants::new();
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::South, 10));
        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(wants.split.choose(&mut rng), Some((Direction::North, 0)));
    }

    #[test]
    fn test_really_really_want_choose_strongest_wins2() {
        let mut wants = Wants::new();
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::South, 10));
        wants.split.want((Direction::South, 10));
        wants.split.want((Direction::South, 10));

        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(wants.split.choose(&mut rng), Some((Direction::South, 10)));
    }

    #[test]
    fn test_really_really_want_choose_strongest_wins_with_strength() {
        let mut wants = Wants::new();
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::South, 10));
        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(
            wants.split.choose_with_strength(&mut rng),
            Some(((Direction::North, 0), 2))
        );
    }

    #[test]
    fn test_really_really_want_choose_strongest_wins2_with_strength() {
        let mut wants = Wants::new();
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::North, 0));
        wants.split.want((Direction::South, 10));
        wants.split.want((Direction::South, 10));
        wants.split.want((Direction::South, 10));

        let mut rng = SmallRng::from_seed([0; 32]);

        assert_eq!(
            wants.split.choose_with_strength(&mut rng),
            Some(((Direction::South, 10), 3))
        );
    }

    #[test]
    fn test_get_by_value() {
        let mut wants = Wants::new();
        wants.block_merge.want(Direction::North);
        wants.block_merge.want(Direction::North);
        wants.block_merge.want(Direction::South);
        wants.block_merge.want(Direction::South);
        wants.block_merge.want(Direction::South);

        assert_eq!(
            wants.block_merge.get_strength_by_value(Direction::North),
            Some(2)
        );
        assert_eq!(
            wants.block_merge.get_strength_by_value(Direction::South),
            Some(3)
        );
        assert_eq!(
            wants.block_merge.get_strength_by_value(Direction::West),
            None
        );
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
