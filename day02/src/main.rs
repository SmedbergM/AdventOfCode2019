use std::io;
use std::io::prelude::*;

#[derive(PartialEq, Debug)]
enum StepResult {
    Done,
    Running
}

struct Puzzle {
    memory: Vec<u32>,
    instruction_pointer: usize
}

impl Puzzle {
    fn new() -> Puzzle {
        Puzzle { memory: Vec::new(), instruction_pointer: 0 }
    }

    fn from_str(line: &str) -> Puzzle {
        let mut puzzle = Puzzle::new();
        for x in line.split(",") {
            for x32 in u32::from_str_radix(x, 10) {
                puzzle.push(x32)
            }
        }
        puzzle
    }

    fn push(&mut self, x: u32) {
        self.memory.push(x)
    }

    fn step(&mut self) -> StepResult {
        match self.memory[self.instruction_pointer] {
            1 => self.add_step(),
            2 => self.multiply_step(),
            99 => StepResult::Done,
            other =>
                panic!("Instruction pointer pointed to invalid instruction id {}", other)
        }
    }

    fn add_step(&mut self) -> StepResult {
        let ip = self.instruction_pointer;
        let idx1 = self.memory[ip + 1] as usize;
        let idx2 = self.memory[ip + 2] as usize;
        let idx3 = self.memory[ip + 3] as usize;
        self.memory[idx3] = self.memory[idx1] + self.memory[idx2];
        self.instruction_pointer += 4;
        match self.memory[self.instruction_pointer] {
            1 | 2 => StepResult::Running,
            99 => StepResult::Done,
            other => {
                panic!("Opcode {} does not code a valid operation!", other)
            }
        }
    }

    fn multiply_step(&mut self) -> StepResult {
        let ip = self.instruction_pointer;
        let idx1 = self.memory[ip + 1] as usize;
        let idx2 = self.memory[ip + 2] as usize;
        let idx3 = self.memory[ip + 3] as usize;
        self.memory[idx3] = self.memory[idx1] * self.memory[idx2];
        self.instruction_pointer += 4;
        match self.memory[self.instruction_pointer] {
            1 | 2 => StepResult::Running,
            99 => StepResult::Done,
            other => {
                panic!("Opcode {} does not code a valid operation!", other)
            }
        }
    }

    pub fn run(&mut self) {
        let mut r = StepResult::Running;
        while r != StepResult::Done {
            r = self.step()
        }
    }

    pub fn len(&self) -> usize {
        self.memory.len()
    }

    pub fn head(&self) -> u32 {
        self.memory[0]
    }

    pub fn set(&mut self, noun: u32, verb: u32) {
        self.memory[1] = noun;
        self.memory[2] = verb;
    }
}

impl Clone for Puzzle {
    fn clone(&self) -> Puzzle {
        Puzzle { 
            memory: self.memory.clone(),
            instruction_pointer: self.instruction_pointer.clone()
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut line_iterator = stdin.lock().lines();
    let puzzle = match line_iterator.next().and_then(|maybe_line| maybe_line.ok()) {
        Some(line) => 
            Puzzle::from_str(&line),
        None => {
            eprintln!("Error reading line from stdin!");
            Puzzle::new()
        }
    };

    println!("Puzzle parsed with {} memory", &puzzle.len());
    let mut puzzle_part1 = puzzle.clone();
    puzzle_part1.set(12, 2);
    puzzle_part1.run();
    println!("Part 1: Head == {} after run", &puzzle_part1.head());

    let target = 19690720;
    for noun in 0..100 {
        for verb in 0..100 {
            let mut puzzle_xy = puzzle.clone();
            puzzle_xy.set(noun, verb);
            puzzle_xy.run();
            if puzzle_xy.head() == target {
                println!("Computed target {} with noun/verb {}{}", target, noun, verb);
                break
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_step_test() {
        let mut puzzle = Puzzle::from_str("1,0,0,0,99");
        assert_eq!(puzzle.add_step(), StepResult::Done);
        assert_eq!(puzzle.memory[..], [2,0,0,0,99]);
        assert_eq!(puzzle.instruction_pointer, 4);
    }

    #[test]
    fn multiply_step_test() {
        let mut puzzle = Puzzle::from_str("2,3,0,3,99");
        assert_eq!(puzzle.multiply_step(), StepResult::Done);
        assert_eq!(puzzle.memory[..], [2,3,0,6,99]);
        assert_eq!(puzzle.instruction_pointer, 4);

        let mut puzzle = Puzzle::from_str("2,4,4,5,99,0");
        assert_eq!(puzzle.multiply_step(), StepResult::Done);
        assert_eq!(puzzle.memory[..], [2,4,4,5,99,9801]);
        assert_eq!(puzzle.instruction_pointer, 4);
    }

    #[test]
    fn run_test() {
        let mut puzzle = Puzzle::from_str("1,9,10,3,2,3,11,0,99,30,40,50");
        assert_eq!(puzzle.step(), StepResult::Running);
        assert_eq!(puzzle.instruction_pointer, 4);
        assert_eq!(puzzle.memory[..], [1,9,10,70,2,3,11,0,99,30,40,50]);

        assert_eq!(puzzle.step(), StepResult::Done);
        assert_eq!(puzzle.instruction_pointer, 8);
        assert_eq!(puzzle.memory[..], [3500,9,10,70,2,3,11,0,99,30,40,50]);

        let mut puzzle = Puzzle::from_str("1,1,1,4,99,5,6,0,99");
        assert_eq!(puzzle.step(), StepResult::Running);
        assert_eq!(puzzle.instruction_pointer, 4);
        assert_eq!(puzzle.memory[..], [1,1,1,4,2,5,6,0,99]);

        assert_eq!(puzzle.step(), StepResult::Done);
        assert_eq!(puzzle.instruction_pointer, 8);
        assert_eq!(puzzle.memory[..], [30,1,1,4,2,5,6,0,99]);
    }
}