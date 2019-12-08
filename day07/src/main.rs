use std::io;
use std::io::prelude::*;

use intcode::Program;

struct Permutations {
    k: u8,
    idx_k: Option<usize>, // the index where to place (K-1) in the yielded vector.
    rest: Option<Box<Permutations>> // the permutations of [0..(K-1)]
}

impl Permutations {
    fn new(k: u8) -> Permutations {
        match k {
            0 => Permutations {
                k: 0,
                idx_k: None,
                rest: None
            },
            k => Permutations {
                k: k,
                idx_k: Some((k - 1) as usize),
                rest: Some(Box::new(Permutations::new(k - 1)))
            }
        }
    }
}

impl Iterator for Permutations {
    type Item = Vec<u8>; // should be [u8; self.k] but that's not legal in Rust

    fn next(&mut self) -> Option<Vec<u8>> {
        self.idx_k.and_then(|idx| {
            let mut v = Vec::new();
            if let Some(smaller) = self.rest.as_mut().and_then(|rest| rest.next()) {
                v.extend_from_slice(&smaller[..idx]);
                v.push(self.k - 1);
                v.extend_from_slice(&smaller[idx..]);
                Some(v)
            } else if self.k == 1 {
                v.push(0);
                self.idx_k = None;
                Some(v)
            } else {
                match idx {
                    0 => None,
                    idx => {
                        self.idx_k = Some(idx - 1);
                        self.rest = Some(Box::new(Permutations::new(self.k - 1)));
                        self.next()
                    }
                }
            }
        })
    }
}

fn amp_stack(program: &Program, perm: &Vec<u8>) -> Option<i32> {
    let mut prev_return_code = 0;

    for i in 0..5 {
        let mut inputs = vec!();
        inputs.push(perm[i] as i32);
        inputs.push(prev_return_code);

        let mut amp = program.clone();

        if let Some(r) = amp.run(&inputs[..], &mut |_| {}) {
            prev_return_code = r;
        } else {
            eprintln!("Program did not produce output on input {:?}", &inputs);
            return None
        }
    }

    Some(prev_return_code)
}

fn amp_stack_feeback(program: &Program, perm: &Vec<u8>) -> Option<i32> {
    let mut amps: Vec<Program> = (0..5).map(|i| {
        let mut p = program.clone();
        p.read_input((perm[i] + 5) as i32);
        p
    }).collect();

    let mut last_output = 0;
    for i in std::iter::repeat(0..5).flatten() {
        let amp = &mut amps[i];
        amp.read_input(last_output);
        if let Some(out) = amp.await_output(&mut |_| {}) {
            last_output = out;
        } else {
            return Some(last_output)
        }
    }
    return Some(last_output)
}

fn best_amp_stack(program: &Program) -> i32 {
    let mut m = i32::min_value();
    for perm in Permutations::new(5) {
        if let Some(x) = amp_stack(&program, &perm) {
            if x > m {
                m = x;
                println!("Better value {} found at perm {:?}", x, &perm);
            }
        }
    }
    m
}

fn best_amp_stack_feedback(program: &Program) -> i32 {
    let mut m = i32::min_value();
    for perm in Permutations::new(5) {
        if let Some(x) = amp_stack_feeback(&program, &perm) {
            if x > m {
                m = x;
                println!("Better value {} found at amp settings {:?}", x, &perm);
            }
        }
    }
    m
}

fn read_one_line_from_stdin() -> String {
    let stdin = io::stdin();
    let line = stdin.lock().lines().next().unwrap().unwrap();
    line
}

fn main() {
    let line = read_one_line_from_stdin();
    let program = Program::from_str(&line);

    let m = best_amp_stack(&program);
    println!("Set thrusters to {}", m);

    let m = best_amp_stack_feedback(&program);
    println!("On second thought, set thrusters to {}", m);

}

#[cfg(test)]
mod permutation_tests {
    use super::*;

    #[test]
    fn permutations_0_spec() {
        let mut perms = Permutations::new(0);
        assert_eq!(perms.next(), None)
    }

    #[test]
    fn permutations_1_spec() {
        let mut perms = Permutations::new(1);
        assert_eq!(perms.next(), Some(vec!(0)));
        assert_eq!(perms.next(), None);
    }

    #[test]
    fn permutation_2_spec() {
        let mut perms = Permutations::new(2);
        assert_eq!(perms.next(), Some(vec!(0,1)));
        assert_eq!(perms.next(), Some(vec!(1,0)));
        assert_eq!(perms.next(), None);
    }

    #[test]
    fn permutations_3_spec() {
        let mut perms = Permutations::new(3);
        assert_eq!(perms.next(), Some(vec!(0,1,2)));
        assert_eq!(perms.next(), Some(vec!(1,0,2)));
        assert_eq!(perms.next(), Some(vec!(0,2,1)));
        assert_eq!(perms.next(), Some(vec!(1,2,0)));
        assert_eq!(perms.next(), Some(vec!(2,0,1)));
        assert_eq!(perms.next(), Some(vec!(2,1,0)));
        assert_eq!(perms.next(), None);
    }

    #[test]
    fn permutations_5_spec() {
        let p5 = Permutations::new(5).fold(0, |a, _| a + 1);
        assert_eq!(p5, 120);
    }
}

#[cfg(test)]
mod amp_stack_tests {
    use super::*;

    #[test]
    fn amp_stack_1() {
        let program = Program::from_str("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0");
        let m = best_amp_stack(&program);
        assert_eq!(m, 43210);
    }

    #[test]
    fn amp_stack_2() {
        let program = Program::from_str("3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0");
        let m = best_amp_stack(&program);
        assert_eq!(m, 54321);
    }

    #[test]
    fn amp_stack_3() {
        let program = Program::from_str("3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0");
        let m = best_amp_stack(&program);
        assert_eq!(m, 65210);
    }

    #[test]
    fn amp_stack_1_feedback() {
        let program = Program::from_str("3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5");
        let perm = vec!(4,3,2,1,0);
        let output = amp_stack_feeback(&program, &perm);
        assert_eq!(output, Some(139629729));

        let best_output = best_amp_stack_feedback(&program);
        assert_eq!(output.unwrap(), best_output);
    }

    #[test]
    fn amp_stack_2_feedback() {
        let program = Program::from_str("3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10");
        let perm = vec!(4,2,3,0,1);
        let output = amp_stack_feeback(&program, &perm);
        assert_eq!(output, Some(18216));

        let best_output = best_amp_stack_feedback(&program);
        assert_eq!(output.unwrap(), best_output);
    }
}