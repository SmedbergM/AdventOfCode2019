use std::io;
use std::io::prelude::*;

fn fuel(m: &u32) -> u32 {
    match m/3 {
        0 | 1 | 2 => 0,
        other => other - 2
    }
}

struct RocketModule(u32);

impl RocketModule {
    fn from_line(line: &str) -> Option<RocketModule> {
        u32::from_str_radix(line, 10).ok()
            .map(|weight| RocketModule(weight))
    }

    fn weight(&self) -> u32 {
        self.0
    }

    fn fuel_naive(&self) -> u32 {
        fuel(&self.weight())
    }

    fn cumulative_fuel(&self) -> u32 {
        let mut cf = self.fuel_naive();
        let mut fuel_delta = fuel(&cf);
        while fuel_delta > 0 {
            cf += fuel_delta;
            fuel_delta = fuel(&fuel_delta);
        }
        cf
    }
}

struct Puzzle {
    modules: Vec<RocketModule>
}

impl Puzzle {
    fn new() -> Puzzle {
        Puzzle { modules: Vec::new() }
    }

    fn push(&mut self, module: RocketModule) {
        self.modules.push(module)
    }

    fn fuel_naive(&self) -> u32 {
        self.modules.iter().fold(0, |acc, module| acc + module.fuel_naive())
    }

    fn cumulative_fuel(&self) -> u32 {
        self.modules.iter().fold(0, |acc, module| acc + module.cumulative_fuel())
    }
}

fn main() {
    let stdin = io::stdin();
    let mut puzzle = Puzzle::new();
    for maybe_line in stdin.lock().lines() {
        match maybe_line.ok().and_then(|line| RocketModule::from_line(&line)) {
            Some(module) => puzzle.push(module),
            None => eprintln!("Encountered error reading a line...")
        }
    }
    let part1 = puzzle.fuel_naive();
    println!("Part 1: Total fuel {}.", part1);

    let part2 = puzzle.cumulative_fuel();
    println!("Part 2: Total fuel {}", part2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_spec() {
        assert_eq!(fuel(&3),0);
    }

    #[test]
    fn fuel_naive_spec() {
        let module = RocketModule(12);
        assert_eq!(module.fuel_naive(), 2);
        let module = RocketModule(14);
        assert_eq!(module.fuel_naive(), 2);
        let module = RocketModule(1969);
        assert_eq!(module.fuel_naive(), 654);
        let module = RocketModule(100756);
        assert_eq!(module.fuel_naive(), 33583);
    }

    #[test]
    fn cumulative_fuel_spec() {
        let module = RocketModule(12);
        assert_eq!(module.cumulative_fuel(), 2);

        let module = RocketModule(1969);
        assert_eq!(module.cumulative_fuel(), 966);

        let module = RocketModule(100756);
        assert_eq!(module.cumulative_fuel(), 50346)
    }
}