use std::convert::{TryFrom, Into};
use std::char;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use intcode::{Program, State};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct XY {
    x: usize, y: usize
}

impl XY {
    fn new(x: usize, y: usize) -> XY {
        XY { x, y }
    }

    fn north(&self) -> Option<XY> {
        usize::checked_sub(self.y, 1).map(|y| XY { x: self.x, y })
    }

    fn west(&self) -> Option<XY> {
        usize::checked_sub(self.x, 1).map(|x| XY { y: self.y, x })
    }

    fn south(&self) -> XY {
        XY { x: self.x, y: self.y + 1 }
    }

    fn east(&self) -> XY {
        XY { x: self.x + 1, y: self.y }
    }

}

struct Scaffold {
    p: BTreeMap<XY, char>
}

impl Scaffold {
    pub fn alignment_checksum(&self) -> usize {
        self.crossings().iter().fold(0, |acc, xy| acc + (xy.x * xy.y))
    }

    fn crossings(&self) -> BTreeSet<&XY> {
        self.p.keys().filter(|xy| self.is_crossing(xy)).collect()
    }

    fn is_crossing(&self, xy: &XY) -> bool {
        match self.p.get(xy) {
            None => false,
            Some('#') => {
                xy.north().and_then(|xyn| {
                    xy.west().and_then(|xyw| {
                        self.p.get(&xyn).filter(|&&c| c == '#').and_then(|_| {
                            self.p.get(&xyw).filter(|&&c| c == '#')
                        })
                    })
                }).and_then(|_| {
                    self.p.get(&xy.south()).filter(|&&c| c == '#')
                }).and_then(|_| {
                    self.p.get(&xy.east()).filter(|&&c| c == '#')
                }).map(|_| true).unwrap_or(false)
            },
            _ => false
        }
    }
}

impl fmt::Display for Scaffold {
    fn fmt(&self, writer: &mut fmt::Formatter) -> fmt::Result {
        let (xmax, ymax) = self.p.iter().fold((0,0), |(xmax, ymax), (k, _)| {
            (usize::max(k.x, xmax), usize::max(k.y, ymax))
        });
        let mut s = String::new();
        for y in 0..=ymax {
            for x in 0..=xmax {
                match self.p.get(&XY::new(x,y)) {
                    None => s.push(' '),
                    Some(c) => s.push(*c)
                }
            }
            s.push('\n');
        }
        write!(writer, "{}", s)
    }
}

fn read_ascii(program: &mut Program) -> Scaffold {
    let mut p = BTreeMap::new();
    let mut x = 0;
    let mut y = 0;

    while let State::Output(c64) = program.await_output() {
        for c32 in u32::try_from(c64) {
            match char::from_u32(c32) {
                None => {
                    eprintln!("{} is not a valid character!", c32);
                    x += 1;
                },
                Some('\n') => {
                    x = 0;
                    y += 1;
                },
                Some(c) => {
                    p.insert(XY::new(x,y), c);
                    x += 1;
                }
            }
        }
    }
    Scaffold { p }
}

fn collect_dust(program: &mut Program, mmr: &str, a: &str, b: &str, c: &str) -> Option<i64> {
    program.overwrite_memory(0, 2);
    for chr in mmr.chars() {
        program.read_input(chr as i64);
    }
    program.read_input('\n' as i64);
    for chr in a.chars() {
        program.read_input(chr as i64);
    }
    program.read_input('\n' as i64);
    for chr in b.chars() {
        program.read_input(chr as i64);
    }
    program.read_input('\n' as i64);
    for chr in c.chars() {
        program.read_input(chr as i64);
    }
    program.read_input('\n' as i64);
    program.read_input('n' as i64);
    program.read_input('\n' as i64);

    while let State::Output(out) = program.await_output() {
        match as_ascii(out) {
            Some(c) => print!("{}", c as char),
            None => {
                println!("");
                println!("Non-character output: {}", out);
                return Some(out)
            }
        }
    };
    return None
}

fn as_ascii(x: i64) -> Option<u8> {
    u8::try_from(x).ok().filter(|u| u.is_ascii())
}

fn main() {
    let puzzle = util::read_single_line_from_stdin().unwrap();
    let program = Program::from_str(&puzzle);
    let mut program1 = program.clone();
    let map = read_ascii(&mut program1);
    println!("{}", &map);
    let ac = map.alignment_checksum();
    println!("Alignment checksum: {}", &ac);

    // cheating here: I solved the second part by hand after printing the scaffolding to STDOUT, then piped my solution in
    let mut program2 = program.clone();
    let main_movement_routine = util::read_single_line_from_stdin().unwrap();
    let movement_a = util::read_single_line_from_stdin().unwrap();
    let movement_b = util::read_single_line_from_stdin().unwrap();
    let movement_c = util::read_single_line_from_stdin().unwrap();

    let dust = collect_dust(&mut program2, &main_movement_routine, &movement_a, &movement_b, &movement_c).unwrap();
    println!("Dust collected: {}", dust);
}
