use std::io;
use std::io::prelude::*;
use std::collections::{HashSet, HashMap};
use std::fmt;


#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Rock {
    x: usize,
    y: usize
}

impl Rock {
    fn is_collinear(&self, r2: &Rock, r3: &Rock) -> bool {
        (self.x*r2.y + r2.x*r3.y + r3.x*self.y) == (self.y*r2.x + r2.y*r3.x + r3.y*self.x)
    }
}

mod asteroid_belt_iter {
    use super::{Rock, AsteroidBelt};
    use std::collections::HashSet;
    use num::integer;
    use std::cmp::Ordering;
    use std::f32::NAN;

    pub struct SouthEastIter<'a> { // iterator over the rocks southeast of a given rock
        base: &'a Rock,
        asteroids: &'a AsteroidBelt,
        point_x: usize,
        point_y: usize
    }

    impl SouthEastIter<'_> {
        pub fn new<'a>(base: &'a Rock, asteroids: &'a AsteroidBelt) -> SouthEastIter<'a> {
            SouthEastIter {
                base,
                asteroids,
                point_x: base.x + 1,
                point_y: base.y
            }
        }
    }

    impl Iterator for SouthEastIter<'_> {
        type Item = Rock;

        fn next(&mut self) -> Option<Rock> {
            if self.point_y >= self.asteroids.rocks.len() {
                None
            } else if self.point_x < self.base.x {
                self.point_x = self.base.x;
                self.next()
            } else {
                let ref rocks = self.asteroids.rocks[self.point_y];
                if self.point_x >= rocks.len() {
                    self.point_x = self.base.x;
                    self.point_y += 1;
                    self.next()
                } else if let Some(true) = rocks.get(self.point_x) {
                    let rock = Rock { y: self.point_y, x: self.point_x };
                    self.point_x += 1;
                    Some(rock)
                } else {
                    self.point_x += 1;
                    self.next()
                }
            }
        }
    }

    pub struct SouthWestIter<'a> {
        base: &'a Rock,
        asteroids: &'a AsteroidBelt,
        point_x: usize,
        point_y: usize
    }

    impl SouthWestIter<'_> {
        pub fn new<'a>(base: &'a Rock, asteroids: &'a AsteroidBelt) -> SouthWestIter<'a> {
            let (point_x, point_y) = match base.x {
                0 => (0, base.y + 1),
                x => (x - 1, base.y)
            };
            SouthWestIter {
                base, asteroids, point_x, point_y
            }
        }
    }

    impl Iterator for SouthWestIter<'_> {
        type Item = Rock;

        fn next(&mut self) -> Option<Rock> {
            if self.point_y >= self.asteroids.rocks.len() {
                None
            } else {
                let ref row = self.asteroids.rocks[self.point_y];
                match self.point_x {
                    0 => {
                        if let Some(true) = row.get(self.point_x) {
                            let rock = Rock { x: self.point_x, y: self.point_y };
                            self.point_x = self.base.x;
                            self.point_y += 1;
                            Some(rock)
                        } else {
                            self.point_x = self.base.x;
                            self.point_y += 1;
                            self.next()
                        }
                    },
                    point_x => {
                        if let Some(true) = row.get(point_x) {
                            let rock = Rock { x: point_x, y: self.point_y};
                            self.point_x -= 1;
                            Some(rock)
                        } else {
                            self.point_x -= 1;
                            self.next()
                        }
                    }
                }
            }
        }
    }

    pub struct AsteroidBeltIterator<'a> {
        asteroids: &'a AsteroidBelt,
        point_x: usize,
        point_y: usize
    }

    impl AsteroidBeltIterator<'_> {
        pub fn new<'a>(asteroids: &'a AsteroidBelt) -> AsteroidBeltIterator<'a> {
            let point_x = 0;
            let point_y = 0;
            AsteroidBeltIterator {
                asteroids, point_x, point_y
            }
        }
    }

    impl Iterator for AsteroidBeltIterator<'_> {
        type Item = Rock;

        fn next(&mut self) -> Option<Rock> {
            self.asteroids.rocks.get(self.point_y).and_then(|row| {
                if self.point_x >= row.len() {
                    self.point_y += 1;
                    self.point_x = 0;
                    self.next()
                } else if let Some(true) = row.get(self.point_x) {
                    let rock = Rock { x: self.point_x, y: self.point_y };
                    self.point_x += 1;
                    Some(rock)
                } else {
                    self.point_x += 1;
                    self.next()
                }
            })
        }
    }

    pub enum Quadrant {
        North,
        NorthEast,
        East,
        SouthEast,
        South,
        SouthWest,
        West,
        NorthWest
    }

    impl Quadrant {
        fn signum(&self) -> (i32, i32) {
            match self {
                Quadrant::North => (0,-1),
                Quadrant::NorthEast => (1,-1),
                Quadrant::East => (1, 0),
                Quadrant::SouthEast => (1, 1),
                Quadrant::South => (0, 1),
                Quadrant::SouthWest => (-1, 1),
                Quadrant::West => (-1, 0),
                Quadrant::NorthWest => (-1, -1)
            }
        }

        fn rotate(&self) -> Quadrant {
            match self {
                Quadrant::North => Quadrant::NorthEast,
                Quadrant::NorthEast => Quadrant::East,
                Quadrant::East => Quadrant::SouthEast,
                Quadrant::SouthEast => Quadrant::South,
                Quadrant::South => Quadrant::SouthWest,
                Quadrant::SouthWest => Quadrant::West,
                Quadrant::West => Quadrant::NorthWest,
                Quadrant::NorthWest => Quadrant::North
            }
        }

        fn slope(&self, p: &(usize, usize)) -> f32 {
            match self {
                Quadrant::North | Quadrant::South => NAN,
                Quadrant::East | Quadrant::West => 0.0,
                _ if p.1 == 0 => NAN,
                Quadrant::SouthEast | Quadrant::NorthWest => {
                    (p.0 as f32) / (p.1 as f32)
                },
                Quadrant::NorthEast | Quadrant::SouthWest => {
                    -(p.0 as f32) / (p.1 as f32)
                }
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum Direction {
        North,
        NorthEast{ dx: usize, dy: usize },
        East,
        SouthEast{ dx: usize, dy: usize },
        South,
        SouthWest { dx: usize, dy: usize },
        West,
        NorthWest { dx: usize, dy: usize }
    }

    impl Direction {
        fn new(quadrant: &Quadrant, p: &(usize, usize)) -> Direction {
            match quadrant {
                Quadrant::North => Direction::North,
                Quadrant::East => Direction::East,
                Quadrant::South => Direction::South,
                Quadrant::West => Direction::West,
                Quadrant::NorthEast => Direction::NorthEast { dx: p.0, dy: p.1 },
                Quadrant::SouthEast => Direction::SouthEast { dx: p.0, dy: p.1 },
                Quadrant::SouthWest => Direction::SouthWest { dx: p.0, dy: p.1 },
                Quadrant::NorthWest => Direction::NorthWest { dx: p.0, dy: p.1 }
            }
        }

        pub fn shift(&self, p: &(usize, usize)) -> Option<(usize, usize)> {
            match self {
                Direction::East => Some((p.0 + 1, p.1)),
                Direction::South => Some((p.0, p.1 + 1)),
                Direction::SouthEast { dx, dy } => Some((p.0 + dx, p.1 + dy)),
                Direction::North => p.1.checked_sub(1).map(|y| (p.0, y)),
                Direction::West => p.0.checked_sub(1).map(|x| (x, p.1)),
                Direction::NorthEast { dx, dy } => {
                    p.1.checked_sub(*dy).map(|y| (p.0 + dx, y))
                },
                Direction::SouthWest { dx, dy } => {
                    p.0.checked_sub(*dx).map(|x| (x, p.1 + dy))
                },
                Direction::NorthWest { dx, dy } => {
                    p.0.checked_sub(*dx).and_then(|x| p.1.checked_sub(*dy).map(|y| (x,y)))
                }
            }
        }
    }

    pub struct SlopeIterator {
        quadrant: Quadrant,
        vs: HashSet<(usize, usize)>,
        xmin: usize,
        xmax: usize,
        ymin: usize,
        ymax: usize
    }

    impl SlopeIterator {
        pub fn new(base: (usize, usize), width: usize, height: usize) -> SlopeIterator {
            let quadrant = Quadrant::North;
            let mut vs = HashSet::new();
            vs.insert((0,1));
            // for a base of (a,b) in a field of (0..width)x(0..height), x increasing to the east, y increasing south,
            // if we shift everything by (-a, -b) we get
            // base = (0,0)
            // upper left corner = (-a, -b)
            // lower right corner = (width - a, height - b)
            let xmin = base.0;
            let xmax = width - base.0;
            let ymin = base.1;
            let ymax = height - base.1;
            SlopeIterator {
                quadrant, vs, xmin, xmax, ymin, ymax
            }
        }

        pub fn base(&self) -> Rock {
            Rock { x: self.xmin, y: self.ymin }
        }

        pub fn width(&self) -> usize {
            self.xmin + self.xmax
        }

        pub fn height(&self) -> usize {
            self.ymin + self.ymax
        }

        fn quadrant_directions(&self, quadrant: &Quadrant) -> HashSet<(usize, usize)> {
            let mut directions = HashSet::new();

            match quadrant {
                Quadrant::North | Quadrant::South => {
                    directions.insert((0,1));
                },
                Quadrant::East | Quadrant::West => {
                    directions.insert((1,0));
                },
                _ => {
                    let signum = quadrant.signum();
                    let xrange = match signum.0 {
                        1 => 1..self.xmax,
                        -1 => 1..(self.xmin + 1),
                        _ => panic!()
                    };
                    let yrange = match signum.1 {
                        1 => 1..self.ymax,
                        -1 => 1..(self.ymin + 1),
                        _ => panic!()
                    };
                    for x in xrange.clone() {
                        for y in yrange.clone() {
                            if integer::gcd(x, y) == 1 {
                                directions.insert((x,y));
                            }
                        }
                    }
                }
            }

            directions
        }
    }

    impl Iterator for SlopeIterator {
        type Item = Direction;

        fn next(&mut self) -> Option<Direction> {
            let opt_v = self.vs.iter().max_by(|p1, p2| {
                self.quadrant.slope(p1).partial_cmp(&self.quadrant.slope(p2)).unwrap_or(Ordering::Equal)
            }).map(|v| v.clone());
            if let Some(v) = opt_v {
                self.vs.remove(&v);
                Some(Direction::new(&self.quadrant, &v))
            } else {
                self.quadrant = self.quadrant.rotate();
                self.vs = self.quadrant_directions(&self.quadrant);
                self.next()
            }
        }
    }
}

pub struct AsteroidBelt {
    rocks: Vec<Vec<bool>>
}

impl AsteroidBelt {
    fn new() -> AsteroidBelt {
        AsteroidBelt { rocks: Vec::new() }
    }

    fn add_row(&mut self, line: &str) {
        let row = line.chars().map(|c| c == '#').collect();
        self.rocks.push(row);
    }

    fn size(&self) -> usize {
        self.rocks.iter().fold(0, |acc, rs| {
            rs.iter().fold(acc, |acc2, b| acc2 + (*b as usize))
        })
    }

    fn nonempty(&self) -> bool {
        for rs in &self.rocks {
            for b in rs {
                if *b {
                    return true
                }
            }
        };
        return false
    }

    fn iter<'a>(&'a self) -> asteroid_belt_iter::AsteroidBeltIterator<'a> {
        asteroid_belt_iter::AsteroidBeltIterator::new(self)
    }

    fn se<'a>(&'a self, base: &'a Rock) -> asteroid_belt_iter::SouthEastIter {
        asteroid_belt_iter::SouthEastIter::new(base, self)
    }

    fn sw<'a>(&'a self, base: &'a Rock) -> asteroid_belt_iter::SouthWestIter {
        asteroid_belt_iter::SouthWestIter::new(base, self)
    }

    fn count_obstructed_all(&self) -> HashMap<Rock, usize> {
        let mut obstruct_store: HashMap<Rock, HashSet<Rock>> = HashMap::new();

        for r1 in self.iter() {
            for r2 in self.sw(&r1) {
                for r3 in self.sw(&r2) {
                    if r1.is_collinear(&r2, &r3) {
                        obstruct_store.entry(r1).or_insert(HashSet::new()).insert(r3);
                        obstruct_store.entry(r3).or_insert(HashSet::new()).insert(r1);
                    }
                }
            }
            for r2 in self.se(&r1) {
                if r2.x > r1.x && r2.y > r1.y { // don't double-count obstructions on the vertical/horizontal
                    for r3 in self.se(&r2) {
                        if r1.is_collinear(&r2, &r3) {
                            obstruct_store.entry(r1).or_insert(HashSet::new()).insert(r3);
                            obstruct_store.entry(r3).or_insert(HashSet::new()).insert(r1);
                        }
                    }
                }
            }
        }
        
        obstruct_store.iter().map(|(&rock, others)| {
            (rock, others.len())
        }).collect()
    }

    fn least_obstructed(&self) -> (Rock, usize) {
        let obs = self.count_obstructed_all();
        let (best_rock, best_rock_obstructed) = obs.iter().min_by_key(|(_, &c)| c).unwrap();
        (*best_rock, self.size() - best_rock_obstructed - 1)
    }

    fn directions(&self, base: &Rock) -> asteroid_belt_iter::SlopeIterator {
        let ymax = self.rocks.len();
        let xmax = self.rocks.iter().fold(0, |m, rs| usize::max(m, rs.len()));
        asteroid_belt_iter::SlopeIterator::new((base.x, base.y), xmax, ymax)
    }

    fn zap(&mut self,directions: &mut asteroid_belt_iter::SlopeIterator) -> Option<(usize, usize)> {
        let xmax = directions.width();
        let ymax = directions.height();
        let base = directions.base();
        for dir in directions {
            let mut target = (base.x, base.y);
            while let Some(p) = dir.shift(&target) {
                if p.0 <= xmax && p.1 <= ymax {
                    target = p;
                    if let Some(true) = self.rocks.get(target.1).and_then(|vs| vs.get(target.0)) {
                        // a hit!
                        self.rocks[target.1][target.0] = false;
                        return Some(target)
                    }
                } else {
                    break
                }
            };
            if self.nonempty() {
                // continue
            } else {
                return None
            }
        };
        return None // actually dead code, but never mind that...
    }
}


impl fmt::Display for AsteroidBelt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {        
        let mut repr = String::new();
        for row in &self.rocks {
            for b in row {
                if *b {
                    repr.push('#');
                } else {
                    repr.push('.');
                }
            }
            repr.push('\n');
        }
        repr.pop();
        write!(formatter, "{}", repr)
    }
}

fn main() {
    let mut asteroids = AsteroidBelt::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines().flat_map(|ma| ma.ok()) {
        asteroids.add_row(&line);
    }
    println!("Hello, asteroids!");
    println!("{}", &asteroids);

    let (best_rock, c) = asteroids.least_obstructed();
    println!("{:?} can see {} other rocks", &best_rock, c);

    let mut k = 1;
    let mut dirs = asteroids.directions(&best_rock);
    while let Some((vx, vy)) = asteroids.zap(&mut dirs) {
        println!("{}: Zapped ({},{})", k, vx, vy);
        k += 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn size_spec() {
        let puzzle_input = ".#..#
                            .....
                            #####
                            ....#
                            ...##";

        let mut asteroids = AsteroidBelt::new();
        for line in puzzle_input.split_whitespace() {
            asteroids.add_row(line);
        }

        assert_eq!(asteroids.size(), 10);
    }

    #[test]
    fn southeast_spec() {
        let puzzle_input = ".#..#
                            .....
                            #####
                            ....#
                            ...##";

        let mut asteroids = AsteroidBelt::new();
        for line in puzzle_input.split_whitespace() {
            asteroids.add_row(line);
        }

        let base = Rock { y: 2, x: 3 };
        let se = asteroid_belt_iter::SouthEastIter::new(&base, &asteroids);
        let mut rocks_expected: HashSet<Rock> = HashSet::new();
        for rock in &[
            Rock {x: 4, y: 2},
            Rock {y: 3, x: 4},
            Rock {y: 4, x: 3},
            Rock {y: 4, x: 4}
        ] {
            rocks_expected.insert(*rock);
        }
        let rocks_actual: HashSet<Rock> = se.collect();
        assert_eq!(rocks_actual, rocks_expected)
    }

    #[test]
    fn southwest_spec() {
        let puzzle_input = ".#..#
                            .....
                            #####
                            ....#
                            ...##";

        let mut asteroids = AsteroidBelt::new();
        for line in puzzle_input.split_whitespace() {
            asteroids.add_row(line);
        }

        let base = Rock { y: 0, x: 1 };
        let sw = asteroid_belt_iter::SouthWestIter::new(&base, &asteroids);
        let mut rocks_expected: HashSet<Rock> = HashSet::new();
        for rock in &[
            Rock {x: 0, y: 2},
            Rock {x: 1, y: 2}
        ] {
            rocks_expected.insert(*rock);
        }
        let rocks_actual: HashSet<Rock> = sw.collect();
        assert_eq!(rocks_actual, rocks_expected);

    }

    #[test]
    fn obstruct_test_1() {
        let puzzle_input = ".#..#
                            .....
                            #####
                            ....#
                            ...##";

        let mut asteroids = AsteroidBelt::new();
        for line in puzzle_input.split_whitespace() {
            asteroids.add_row(line);
        }

        let (lo, c) = asteroids.least_obstructed();
        assert_eq!(lo, Rock { x: 3, y: 4 });
        assert_eq!(c, 8);
    }

    #[test]
    fn obstruct_test_2() {
        let puzzle_input = "......#.#.
                            #..#.#....
                            ..#######.
                            .#.#.###..
                            .#..#.....
                            ..#....#.#
                            #..#....#.
                            .##.#..###
                            ##...#..#.
                            .#....####";
        let mut asteroids = AsteroidBelt::new();
        for line in puzzle_input.split_whitespace() {
            asteroids.add_row(line);
        }
        let (best, c) = asteroids.least_obstructed();
        assert_eq!(best, Rock { x: 5, y: 8 });
        assert_eq!(c, 33);
    }

    #[test]
    fn obstruct_test_3() {
        let puzzle_input = "#.#...#.#.
                            .###....#.
                            .#....#...
                            ##.#.#.#.#
                            ....#.#.#.
                            .##..###.#
                            ..#...##..
                            ..##....##
                            ......#...
                            .####.###.";
        let mut asteroids = AsteroidBelt::new();
        for line in puzzle_input.split_whitespace() {
            asteroids.add_row(line);
        }
        let (best, c) = asteroids.least_obstructed();
        assert_eq!(best, Rock { x: 1, y: 2 });
        assert_eq!(c, 35);
    }

    #[test]
    fn obstruct_test_4() {
        let puzzle_input = ".#..#..###
                            ####.###.#
                            ....###.#.
                            ..###.##.#
                            ##.##.#.#.
                            ....###..#
                            ..#.#..#.#
                            #..#.#.###
                            .##...##.#
                            .....#.#..";
        let mut asteroids = AsteroidBelt::new();
        for line in puzzle_input.split_whitespace() {
            asteroids.add_row(line);
        }
        let (best, c) = asteroids.least_obstructed();
        assert_eq!(best, Rock { x: 6, y: 3 });
        assert_eq!(c, 41);
    }

    #[test]
    fn obstruct_test_5() {
        let puzzle_input = ".#..##.###...#######
                            ##.############..##.
                            .#.######.########.#
                            .###.#######.####.#.
                            #####.##.#.##.###.##
                            ..#####..#.#########
                            ####################
                            #.####....###.#.#.##
                            ##.#################
                            #####.##.###..####..
                            ..######..##.#######
                            ####.##.####...##..#
                            .#####..#.######.###
                            ##...#.##########...
                            #.##########.#######
                            .####.#.###.###.#.##
                            ....##.##.###..#####
                            .#.#.###########.###
                            #.#.#.#####.####.###
                            ###.##.####.##.#..##";
        let mut asteroids = AsteroidBelt::new();
        for line in puzzle_input.split_whitespace() {
            asteroids.add_row(line);
        }
        let (best, c) = asteroids.least_obstructed();
        assert_eq!(best, Rock { x: 11, y: 13 });
        assert_eq!(c, 210);
    }

    #[test]
    fn zap_test() {
        let puzzle = 
        ".#....#####...#..
         ##...##.#####..##
         ##...#...#.#####.
         ..#.....#...###..
         ..#.#.....#....##";
        let mut asteroids = AsteroidBelt::new();
        for line in puzzle.split_whitespace() {
            asteroids.add_row(line);
        }

        let base = Rock { x: 8, y: 3 };
        let mut dirs = asteroids.directions(&base);

        assert_eq!(asteroids.zap(&mut dirs), Some(( 8, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 9, 0)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 9, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some((10, 0)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 9, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some((11, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some((12, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some((11, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some((15, 1)));

        assert_eq!(asteroids.zap(&mut dirs), Some((12, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some((13, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some((14, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some((15, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some((12, 3)));
        assert_eq!(asteroids.zap(&mut dirs), Some((16, 4)));
        assert_eq!(asteroids.zap(&mut dirs), Some((15, 4)));
        assert_eq!(asteroids.zap(&mut dirs), Some((10, 4)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 4, 4)));

        assert_eq!(asteroids.zap(&mut dirs), Some(( 2, 4)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 2, 3)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 0, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 1, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 0, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 1, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 5, 2)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 1, 0)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 5, 1)));

        assert_eq!(asteroids.zap(&mut dirs), Some(( 6, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 6, 0)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 7, 0)));
        assert_eq!(asteroids.zap(&mut dirs), Some(( 8, 0)));
        assert_eq!(asteroids.zap(&mut dirs), Some((10, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some((14, 0)));
        assert_eq!(asteroids.zap(&mut dirs), Some((16, 1)));
        assert_eq!(asteroids.zap(&mut dirs), Some((13, 3)));
        assert_eq!(asteroids.zap(&mut dirs), Some((14, 3)));

    }
}

#[cfg(test)]
mod asteroid_iter_tests {
    use super::asteroid_belt_iter::*;
    use super::*;

    #[test]
    fn slope_iter_test() {
        let puzzle = ".#..#
                      .....
                      #####
                      ....#
                      ...##";
        let mut asteroids = AsteroidBelt::new();
        for line in puzzle.split_whitespace() {
            asteroids.add_row(line);
        }
        let rock = Rock { x: 2, y: 2 };
        let mut slopes = asteroids.directions(&rock);
        assert_eq!(slopes.next(), Some(Direction::North));
        assert_eq!(slopes.next(), Some(Direction::NorthEast { dx: 1, dy: 2 }));
        assert_eq!(slopes.next(), Some(Direction::NorthEast { dx: 1, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::NorthEast { dx: 2, dy: 1 }));

        assert_eq!(slopes.next(), Some(Direction::East));
        assert_eq!(slopes.next(), Some(Direction::SouthEast { dx: 2, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::SouthEast { dx: 1, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::SouthEast { dx: 1, dy: 2 }));

        assert_eq!(slopes.next(), Some(Direction::South));
        assert_eq!(slopes.next(), Some(Direction::SouthWest { dx: 1, dy: 2 }));
        assert_eq!(slopes.next(), Some(Direction::SouthWest { dx: 1, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::SouthWest { dx: 2, dy: 1 }));

        assert_eq!(slopes.next(), Some(Direction::West));
        assert_eq!(slopes.next(), Some(Direction::NorthWest { dx: 2, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::NorthWest { dx: 1, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::NorthWest { dx: 1, dy: 2 }));

        assert_eq!(slopes.next(), Some(Direction::North));

        let rock = Rock { x: 3, y: 2 };
        let mut slopes = asteroids.directions(&rock);
        assert_eq!(slopes.next(), Some(Direction::North));

        assert_eq!(slopes.next(), Some(Direction::NorthEast { dx: 1, dy: 2 }));
        assert_eq!(slopes.next(), Some(Direction::NorthEast { dx: 1, dy: 1 }));

        assert_eq!(slopes.next(), Some(Direction::East));

        assert_eq!(slopes.next(), Some(Direction::SouthEast { dx: 1, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::SouthEast { dx: 1, dy: 2 }));

        assert_eq!(slopes.next(), Some(Direction::South));
        assert_eq!(slopes.next(), Some(Direction::SouthWest { dx: 1, dy: 2 }));
        assert_eq!(slopes.next(), Some(Direction::SouthWest { dx: 1, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::SouthWest { dx: 3, dy: 2 }));
        assert_eq!(slopes.next(), Some(Direction::SouthWest { dx: 2, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::SouthWest { dx: 3, dy: 1 }));

        assert_eq!(slopes.next(), Some(Direction::West));
        assert_eq!(slopes.next(), Some(Direction::NorthWest { dx: 3, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::NorthWest { dx: 2, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::NorthWest { dx: 3, dy: 2 }));
        assert_eq!(slopes.next(), Some(Direction::NorthWest { dx: 1, dy: 1 }));
        assert_eq!(slopes.next(), Some(Direction::NorthWest { dx: 1, dy: 2 }));
    }

}