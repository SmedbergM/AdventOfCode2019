use std::collections::BTreeMap;
use std::fmt;

use intcode::{Program, State};

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)]
struct XY {
    x: i32,
    y: i32
}

impl XY {
    fn zero() -> XY {
        XY { x: 0, y: 0 }
    }

    fn step(&self, direction: &Direction) -> XY {
        match direction {
            Direction::North => XY { x: self.x, y: self.y + 1},
            Direction::South => XY { x: self.x, y: self.y - 1},
            Direction::East => XY { x: self.x + 1, y: self.y },
            Direction::West => XY { x: self.x - 1, y: self.y }
        }
    }
}

impl fmt::Display for XY {
    fn fmt(&self, writer: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(writer, "({},{})", self.x, self.y)
    }
}

enum Direction {
    North, South, East, West
}

impl Direction {
    fn all() -> [Direction; 4] {
        [Direction::North, Direction::East, Direction::South, Direction::West]
    }

    fn input_code(&self) -> i64 {
        match self {
            Direction::North => 1,
            Direction::South => 2,
            Direction::West => 3,
            Direction::East => 4
        }
    }
}

enum Square {
    Origin,
    Open,
    Wall,
    Oxygen
}

impl Square {
    fn to_char(&self) -> char {
        match self {
            Square::Origin => '0',
            Square::Open => '.',
            Square::Wall => '#',
            Square::Oxygen => 'T'
        }
    }
}

struct RepairDroid {
    program: Program,
    map: BTreeMap<XY, Square>,
    search_path: Vec<XY>
}

impl RepairDroid {
    fn new(program: Program) -> RepairDroid {
        let mut map = BTreeMap::new();
        map.insert(XY::zero(), Square::Origin);
        RepairDroid { program, map, search_path: vec!(XY::zero()) }
    }

    fn display_map(&self) -> String {
        let (xmin, xmax, ymin, ymax) = self.map.iter().fold((0,0,0,0), |(xmin, xmax, ymin, ymax), (xy, _)| {
            (i32::min(xmin, xy.x), i32::max(xmax, xy.x), i32::min(ymin, xy.y), i32::max(ymax, xy.y))
        });
        let mut s = String::new();
        let mut y = ymax;
        while y >= ymin {
            for x in xmin..=xmax {
                match self.map.get(&XY{ x, y }) {
                    None => s.push(' '),
                    Some(square) => s.push(square.to_char())
                }
            };
            s.push('\n');
            y -= 1
        };
        s.pop();
        s
    }

    fn depth_first_search(&mut self) {
        // basic procedure: look for an unresolved square adjacent to the current position (search_path.last)
        // If none is found, then backtrack one square

        while let Some(current_xy) = self.search_path.last() {
            if let Some(next_direction) = Direction::all().iter()
                .filter(|d| !self.map.contains_key(&current_xy.step(d))).next() {

                self.program.read_input(next_direction.input_code());
                let state = self.program.await_output();
                let output_code = match state {
                    State::Output(code) | State::OutputAwaitingInput(code) => code,
                    _ => {
                        eprintln!("Unexpected state {:?}", state);
                        break
                    }
                };
                match output_code {
                    0 => {
                        self.map.insert(current_xy.step(next_direction), Square::Wall);
                    },
                    1 => {
                        let next_xy = current_xy.step(next_direction);
                        self.map.insert(next_xy.clone(), Square::Open);
                        self.search_path.push(next_xy);
                    },
                    2 => {
                        let next_xy = current_xy.step(next_direction);
                        println!("Found oxygen tank at {} after {} steps", next_xy, self.search_path.len());
                        self.map.insert(next_xy.clone(), Square::Oxygen);
                        self.search_path.push(next_xy);
                    },
                    _ => {
                        eprintln!("Unexpected output {} from program!", output_code);
                        break
                    }
                }
            } else {
                if let Some(current_xy) = self.search_path.pop(){
                    if let Some(previous_xy) = self.search_path.last() {
                        let backtrack_direction = if previous_xy.y > current_xy.y {
                            Direction::North
                        } else if previous_xy.y < current_xy.y {
                            Direction::South
                        } else if previous_xy.x > current_xy.x {
                            Direction::East
                        } else {
                            Direction::West
                        };
                        self.program.read_input(backtrack_direction.input_code());
                        self.program.await_output();
                    }
                }
            }

        }
    }

    fn reoxygenate(&mut self) -> usize {
        let mut steps = 0;
        loop {
            let xys: Vec<XY> = self.map.iter().flat_map(|(xy, square)| {
                match square {
                    Square::Open => {
                        Direction::all().iter().filter(|d| {
                            if let Some(Square::Oxygen) = self.map.get(&xy.step(&d)) {
                                true
                            } else {
                                false
                            }
                        }).next().map(|_| xy.clone())
                    },
                    _ => None
                }
            }).collect();
            if xys.is_empty() {
                break
            } else {
                steps += 1;
                for xy in xys {
                    self.map.insert(xy, Square::Oxygen);
                }
            }
        }
        steps
    }
}

fn main() {
    let puzzle = util::read_single_line_from_stdin().unwrap();
    let program = Program::from_str(&puzzle);
    let mut repair_droid = RepairDroid::new(program);
    repair_droid.depth_first_search();
    println!("{}", repair_droid.display_map());

    let reox_steps = repair_droid.reoxygenate();
    println!("Reoxygenation takes {} steps.", reox_steps);
}
