use std::io;
use std::io::prelude::*;

use std::convert::TryInto;

extern crate regex;
use regex::{Regex, Captures};

struct Puzzle {
    start: u32,
    end:u32
}

impl Puzzle {
    fn from_str(line: &str) -> Option<Puzzle> {
        fn get_u32(cap: &Captures, idx: usize) -> Option<u32> {
            cap.get(idx).and_then(|m| {
                u32::from_str_radix(m.as_str(), 10).ok()
            })
        }
        let pat = Regex::new(r"(\d+)-(\d+)").unwrap();
        pat.captures(line).and_then(|cap| {
            get_u32(&cap, 1).and_then(|start| {
                get_u32(&cap, 2).map(|end| Puzzle{ start, end })
            })
        })
    }

    fn count_passwords(&self) -> usize {
        let mut n = 0;
        for k in self.start..self.end {
            if is_six_digits(&k) && has_adjacent_equal_digits(&k) && is_nondecreasing(&k) {
                n += 1;
            }
        }
        n
    }

    fn count_passwords2(&self) -> usize {
        let mut n = 0;
        for k in self.start..self.end {
            if is_six_digits(&k) && has_adjacent_equal_digits2(&k) && is_nondecreasing(&k) {
                n += 1;
            }
        }
        n
    }
}

struct Digits {
    n: u32
}

impl Iterator for Digits {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        match self.n {
            0 => None,
            ref n => {
                let q = n/10;
                let r = n%10;
                self.n = q;
                r.try_into().ok()
            }
        }
    }
}

fn is_six_digits(n: &u32) -> bool {
    let digits = Digits { n: *n };
    digits.fold(0, |acc, _| acc + 1) == 6
}

fn has_adjacent_equal_digits(n: &u32) -> bool {
    let mut last: Option<u8> = None;
    let digits = Digits { n : *n };
    for d in digits {
        if let Some(r) = last {
            if r == d {
                return true
            }
        };
        last = Some(d)
    };
    false
}

fn has_adjacent_equal_digits2(n: &u32) -> bool { // returns true if n has two consecutive equal digits which are not part of a longer equal substring
    enum LoopState {
        Start,
        PairFound,
        Cons{ d: u8, repeats: usize }
    }

    let digits = Digits { n : *n };
    let final_state: LoopState = digits.fold(LoopState::Start, |state, digit| {
        match state {
            LoopState::Start => LoopState::Cons { d: digit, repeats: 1},
            LoopState::PairFound => LoopState::PairFound,
            LoopState::Cons { d, repeats } if d == digit => LoopState::Cons { d, repeats: repeats + 1 },
            LoopState::Cons { d, repeats: 2} if d != digit => LoopState::PairFound,
            LoopState::Cons { .. } => LoopState::Cons { d: digit, repeats: 1}
        }
    });
    match final_state {
        LoopState::Start => false,
        LoopState::PairFound => true,
        LoopState::Cons { repeats: 2, .. } => true,
        _ => false
    }
}

fn is_nondecreasing(n: &u32) -> bool {
    let digits = Digits { n : *n };
    let mut last: Option<u8> = None;
    for d in digits { // remember, the iteration is from right to left (least significant digit first)
        if let Some(r) = last {
            if r < d {
                return false
            }
        };
        last = Some(d)
    };
    return true
}

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let line = lines.next().unwrap().unwrap();
    let puzzle = Puzzle::from_str(&line).unwrap();
    let part1 = puzzle.count_passwords();
    println!("puzzle ({}, {}) admits {} passwords", &puzzle.start, &puzzle.end, part1);
    let part2 = puzzle.count_passwords2();
    println!("It admits {} passwords with 2-grouping", part2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_digit_spec() {
        let n = 0;
        assert!(!is_six_digits(&n));

        let n = 5;
        assert!(!is_six_digits(&n));

        let n = 12;
        assert!(!is_six_digits(&n));

        let n = 123;
        assert!(!is_six_digits(&n));

        let n = 1234;
        assert!(!is_six_digits(&n));

        let n = 12345;
        assert!(!is_six_digits(&n));

        let n = 123456;
        assert!(is_six_digits(&n));

        let n = 1234567;
        assert!(!is_six_digits(&n));

        let n = 100000;
        assert!(is_six_digits(&n));
    }

    #[test]
    fn adjacent_equal_digits_spec() {
        let n = 5;
        assert!(!has_adjacent_equal_digits(&n));

        let n = 55;
        assert!(has_adjacent_equal_digits(&n));

        let n = 343;
        assert!(!has_adjacent_equal_digits(&n));

        let n = 123321;
        assert!(has_adjacent_equal_digits(&n));
    }

    #[test]
    fn nondecreasing_spec() {
        let n = 12;
        assert!(is_nondecreasing(&n));

        let n = 21;
        assert!(!is_nondecreasing(&n));

        let n = 22;
        assert!(is_nondecreasing(&n));

        let n = 1459;
        assert!(is_nondecreasing(&n));

        let n = 1449;
        assert!(is_nondecreasing(&n));

        let n = 1439;
        assert!(!is_nondecreasing(&n));
    }

    #[test]
    fn adjacent_equal_digits2_spec() {
        let n = 111111;
        assert!(!has_adjacent_equal_digits2(&n));

        let n = 223450;
        assert!(has_adjacent_equal_digits2(&n));

        let n = 112233;
        assert!(has_adjacent_equal_digits2(&n));

        let n = 123444;
        assert!(!has_adjacent_equal_digits2(&n));

        let n = 111122;
        assert!(has_adjacent_equal_digits2(&n));
    }
}