use regex;
use std::io::BufRead;
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Moon {
    x: i32, y: i32, z: i32,
    vx: i32, vy: i32, vz: i32
}

impl Moon {
    fn from_str(line: &str) -> Option<Moon> {
        let opt_pat = regex::Regex::new(r"<x=([0-9\-]+), y=([0-9\-]+), z=([0-9\-]+)>").ok();
        opt_pat.and_then(|pat| {
            pat.captures(line)
        }).and_then(|cap| {
            cap.get(1).and_then(|m1| cap.get(2).and_then(|m2| cap.get(3).and_then(|m3| {
                i32::from_str_radix(m1.as_str(), 10).ok().and_then(|x| {
                    i32::from_str_radix(m2.as_str(), 10).ok().and_then(|y| {
                        i32::from_str_radix(m3.as_str(), 10).ok().map(|z| {
                            Moon {
                                x, y, z,
                                vx: 0, vy: 0, vz: 0
                            }
                        })
                    })
                })
            })))
        })
    }

    fn gravitate(&self, other: &Moon) -> Moon {
        fn delta(ord: Ordering) -> i32 { // returns the value to add to this.x/y/z in case this.x/y/z `ord` other.x/y/z
            match ord {
                Ordering::Equal => 0,
                Ordering::Less => 1,
                Ordering::Greater => -1
            }
        }
        let dx = delta(self.x.cmp(&other.x));
        let dy = delta(self.y.cmp(&other.y));
        let dz = delta(self.z.cmp(&other.z));

        Moon {
            x: self.x, y: self.y, z: self.z,
            vx: self.vx + dx, vy: self.vy + dy, vz: self.vz + dz
        }
    }

    fn vstep(&self) -> Moon {
        Moon { x: self.x + self.vx, y: self.y + self.vy, z: self.z + self.vz,
            vx: self.vx, vy: self.vy, vz: self.vz }
    }

    fn potential_energy(&self) -> i32 {
        i32::abs(self.x) + i32::abs(self.y) + i32::abs(self.z)
    }

    fn kinetic_energy(&self) -> i32 {
        i32::abs(self.vx) + i32::abs(self.vy) + i32::abs(self.vz)
    }

    fn energy(&self) -> i32 {
        self.potential_energy() * self.kinetic_energy()
    }
}

#[derive(Debug, Clone)]
struct Jovian {
    moons: Vec<Moon>
}

impl Jovian {
    fn from_lines<'a, J>(lines: &mut J) -> Jovian
    where J: Iterator<Item=std::io::Result<String>> {
        fn moon_from_result(maybe_line: std::io::Result<String>) -> Option<Moon> {
            maybe_line.ok().and_then(|line| Moon::from_str(&line))
        }
        
        let moons = lines.flat_map(moon_from_result).collect();
        Jovian { moons }
    }

    fn len(&self) -> usize {
        self.moons.len()
    }

    fn tick(&mut self) {
        let mut next_moons = self.moons.clone();
        for i in 0..self.len() {
            for j in (i+1)..self.len() {
                let mi = next_moons[i];
                let mj = next_moons[j];
                let mi2 = mi.gravitate(&mj);
                let mj2 = mj.gravitate(&mi);
                next_moons[i] = mi2;
                next_moons[j] = mj2;
            }
        }

        for i in 0..next_moons.len() {
            let mi = next_moons[i];
            next_moons[i] = mi.vstep();
        }
        self.moons = next_moons;
    }

    fn energy(&self) -> i32 {
        self.moons.iter().map(|m| m.energy()).sum()
    }

    fn find_recurrence(&mut self) -> usize {
        let mut prev_states: HashSet<Vec<Moon>> = HashSet::new();
        loop {
            if prev_states.contains(&self.moons) {
                return prev_states.len()
            } else {
                prev_states.insert(self.moons.clone());
                self.tick();
            }
        }
    }

}

fn main() {
    let stdin = std::io::stdin();
    let mut stdin_locked = stdin.lock().lines();
    let jovian = Jovian::from_lines(&mut stdin_locked);
    println!("Hello, {:?}!", &jovian);

    let mut jovian_part1 = jovian.clone();
    for _ in 0..1000 {
        jovian_part1.tick();
    }
    println!("After 1000 steps, my energy is {}", jovian_part1.energy());
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moon_from_str_spec() {
        let line = "<x=-1, y=0, z=2>";
        let moon = Moon::from_str(&line).unwrap();
        assert_eq!(moon.x, -1);
        assert_eq!(moon.y, 0);
        assert_eq!(moon.z, 2);
        assert_eq!(moon.vx, 0);
        assert_eq!(moon.vy, 0);
        assert_eq!(moon.vz, 0);

        let line = "<x=2, y=-10, z=-7>";
        let moon = Moon::from_str(&line).unwrap();
        assert_eq!(moon.x, 2);
        assert_eq!(moon.y, -10);
        assert_eq!(moon.z, -7);
        assert_eq!(moon.vx, 0);
        assert_eq!(moon.vy, 0);
        assert_eq!(moon.vz, 0);
    }

    #[test]
    fn jovian_from_iter_spec() {
        let puzzle = "<x=-1, y=0, z=2>
        <x=2, y=-10, z=-7>
        <x=4, y=-8, z=8>
        <x=3, y=5, z=-1>";
        let jovian = Jovian::from_lines(&mut puzzle.lines().map(|ll| Ok(String::from(ll))));
        assert_eq!(jovian.len(), 4)
    }

    #[test]
    fn jovian_step_spec() {
        let puzzle = "<x=-1, y=0, z=2>
        <x=2, y=-10, z=-7>
        <x=4, y=-8, z=8>
        <x=3, y=5, z=-1>";
        let mut jovian = Jovian::from_lines(&mut puzzle.lines().map(|ll| Ok(String::from(ll))));

        jovian.tick();

        assert_eq!(jovian.moons[0], Moon { x: 2, y: -1, z: 1, vx: 3, vy: -1, vz: -1});
        assert_eq!(jovian.moons[1], Moon { x: 3, y: -7, z: -4, vx: 1, vy: 3, vz: 3});
        assert_eq!(jovian.moons[2], Moon { x: 1, y: -7, z: 5, vx: -3, vy: 1, vz: -3});
        assert_eq!(jovian.moons[3], Moon { x: 2, y: 2, z: 0, vx: -1, vy: -3, vz: 1});

        jovian.tick();

        assert_eq!(jovian.moons[3], Moon { x: 1, y: -4, z: 2, vx: -1, vy: -6, vz: 2});
        assert_eq!(jovian.moons[2], Moon { x: 1, y: -4, z: -1, vx: 0, vy: 3, vz: -6});
        assert_eq!(jovian.moons[1], Moon { x: 1, y: -2, z: 2, vx: -2, vy: 5, vz: 6});
        assert_eq!(jovian.moons[0], Moon { x: 5, y: -3, z: -1, vx: 3, vy: -2, vz: -2});
    }

    #[test]
    fn moon_energy_spec() {
        let moon0 = Moon { x: 2, y: 1, z: -3, vx: -3, vy: -2, vz: 1 };
        let moon1 = Moon { x: 1, y: -8, z: 0, vx: -1, vy: 1, vz: 3};
        let moon2 = Moon { x: 3, y: -6, z: 1, vx: 3, vy: 2, vz: -3};
        let moon3 = Moon { x: 2, y: 0, z: 4, vx: 1, vy: -1, vz: -1};

        assert_eq!(moon0.energy(), 36);
        assert_eq!(moon1.energy(), 45);
        assert_eq!(moon2.energy(), 80);
        assert_eq!(moon3.energy(), 18);

        let jovian = Jovian {
            moons: vec!(moon0, moon1, moon2, moon3)
        };
        assert_eq!(jovian.energy(), 179);
    }

    #[test]
    fn recurrence_test() {
        let puzzle = "<x=-1, y=0, z=2>
        <x=2, y=-10, z=-7>
        <x=4, y=-8, z=8>
        <x=3, y=5, z=-1>";
        let mut jovian = Jovian::from_lines(&mut puzzle.lines().map(|ll| Ok(String::from(ll))));

        let rc = jovian.find_recurrence();
        assert_eq!(rc, 2772);
    }
}
