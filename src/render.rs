use std::fmt;

extern crate rand;
use rand::distributions::*;
use rand::Rng;
use std::iter::FromIterator;
use std::{thread, time};

// display procedure based off https://oneorten.dev/blog/automata_rust_1/

#[derive(Copy, Clone)]
struct Location {
    resources: u8,
}

const LENGTH: usize = 30;
const WIDTH: usize = 15;

struct World([[Location; LENGTH]; WIDTH]);

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let World(world) = self;

        for row in world.iter() {
            for location in row.iter() {
                let ch = if location.resources > 8 {
                    'X'
                } else if location.resources > 4 {
                    'x'
                } else if location.resources > 2 {
                    '.'
                } else {
                    ' '
                };

                write!(f, "{}", ch)?;
            }

            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl FromIterator<Location> for World {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Location>,
    {
        let mut world = [[Location { resources: 0 }; LENGTH]; WIDTH];
        let mut x = 0;
        let mut y = 0;

        for cell in iter {
            world[x][y] = cell;
            x += 1;
            if x >= WIDTH {
                x = 0;
                y += 1;
                if y >= LENGTH {
                    break;
                }
            }
        }

        World(world)
    }
}

impl Distribution<Location> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Location {
        Location {
            resources: rng.gen_range(0..10),
        }
    }
}

pub fn run() {
    let mut rng = rand::thread_rng();

    print!("\x1b[2J\x1b[?25l");

    let mut world: World = rng.sample_iter(Standard).take(LENGTH * WIDTH).collect();

    loop {
        print!("\x1b[;H");
        println!("{}", world);
        thread::sleep(time::Duration::from_millis(100));
    }
}
