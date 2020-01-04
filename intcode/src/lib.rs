use std::collections::VecDeque;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct Program {
    memory: Vec<i64>,
    instruction_pointer: usize,
    relative_base: i64,
    return_code: Option<i64>,
    input_buffer: VecDeque<i64>
}

impl Program {
    pub fn from_str(line: &str) -> Program {
        let memory: Vec<i64> = line.split(",")
            .flat_map(|s| i64::from_str_radix(s, 10).ok()).collect();
        Program { memory,
            instruction_pointer: 0,
            relative_base: 0,
            return_code: None,
            input_buffer: VecDeque::new()
        }
    }

    fn current_instruction(&self) -> Option<Instruction> {
        match self.memory.get(self.instruction_pointer) {
            None => {
                eprintln!("No instruction found at {}", &self.instruction_pointer);
                None
            },
            Some(x) => Instruction::parse(x)
        }
    }

    fn get(&mut self, idx: usize, mode: &ParameterMode) -> Option<i64> {
        let read_idx = match mode {
            ParameterMode::Immediate => idx,
            ParameterMode::Positional => self.memory[idx] as usize,
            ParameterMode::Relative => self.relative_idx(idx)
        };
        if read_idx >= self.memory.len() {
            self.memory.resize(read_idx + 1, 0);
        }
        self.memory.get(read_idx).map(|x| *x)
    }

    fn relative_idx(&self, idx: usize) -> usize {
        (self.relative_base + self.memory[idx]) as usize
    }

    pub fn read_input(&mut self, input: i64) {
        self.input_buffer.push_back(input)
    }

    fn set(&mut self, idx: usize, value: i64, mode: &ParameterMode) {
        let write_idx = match mode {
            ParameterMode::Positional => self.memory[idx] as usize,
            ParameterMode::Relative => self.relative_idx(idx),
            _ => {
                eprintln!("Setting values in Immediate mode is not supported!");
                idx
            }
        };
        if write_idx >= self.memory.len() {
            self.memory.resize(write_idx + 1, 0);
        }
        self.memory[write_idx] = value;
    }

    fn step(&mut self) -> State {
        enum StepResult {
            Halt,
            Jump,
            Crash,
            Fwd(usize),
            Output(i64)
        }

        fn perform_jump_if(this: &mut Program, nonzero: bool, m1: &ParameterMode, m2: &ParameterMode) -> StepResult {
            match this.get(this.instruction_pointer + 1, m1) {
                None => StepResult::Crash,
                Some(p1) if (p1 != 0) == nonzero => match this.get(this.instruction_pointer + 2, m2) {
                    None => StepResult::Crash,
                    Some(p2) => match usize::try_from(p2) {
                        Err(_) => StepResult::Crash,
                        Ok(p2) => {
                            this.instruction_pointer = p2;
                            StepResult::Jump
                        }
                    }
                },
                _ => StepResult::Fwd(3)
            }
        };
    
        let step_result = match self.current_instruction() {
            None => StepResult::Crash,
            Some(Instruction::Halt) => StepResult::Halt,
            Some(Instruction::Add { m1, m2, m3 }) => {
                let addend1 = self.get(self.instruction_pointer + 1, &m1).unwrap();
                let addend2 = self.get(self.instruction_pointer + 2, &m2).unwrap();
                self.set(self.instruction_pointer + 3, addend1 + addend2, &m3);
                StepResult::Fwd(4)
            },
            Some(Instruction::Mult { m1, m2, m3 }) => {
                let factor1 = self.get(self.instruction_pointer + 1, &m1).unwrap();
                let factor2 = self.get(self.instruction_pointer + 2, &m2).unwrap();
                self.set(self.instruction_pointer + 3, factor1 * factor2, &m3);
                StepResult::Fwd(4)
            },
            Some(Instruction::Input { m1 }) => {
                match self.input_buffer.pop_front() {
                    None => StepResult::Crash,
                    Some(input) => {
                        self.set(self.instruction_pointer + 1, input, &m1);
                        StepResult::Fwd(2)
                    }
                }
            },
            Some(Instruction::Output { m1 }) => {
                match self.get(self.instruction_pointer + 1, &m1) {
                    None => StepResult::Crash,
                    Some(out) => {
                        self.return_code = Some(out);
                        StepResult::Output(out)
                    }
                }
            },
            Some(Instruction::JumpIfTrue { m1, m2 }) => perform_jump_if(self, true, &m1, &m2),
            Some(Instruction::JumpIfFalse { m1, m2 }) => perform_jump_if(self, false, &m1, &m2),
            Some(Instruction::LessThan { m1, m2, m3 }) => {
                match self.get(self.instruction_pointer + 1, &m1).and_then(|p1| self.get(self.instruction_pointer + 2, &m2).map(|p2| (p1, p2))) {
                    None => StepResult::Crash,
                    Some((p1, p2)) => {
                        self.set(self.instruction_pointer + 3, (p1 < p2) as i64, &m3);
                        StepResult::Fwd(4)
                    }
                }
            },
            Some(Instruction::Equals { m1, m2, m3 }) => {
                match self.get(self.instruction_pointer + 1, &m1).and_then(|p1| self.get(self.instruction_pointer + 2, &m2).map(|p2| (p1, p2))) {
                    None => StepResult::Crash,
                    Some((p1, p2)) => {
                        self.set(self.instruction_pointer + 3, (p1 == p2) as i64, &m3);
                        StepResult::Fwd(4)
                    }
                }
            },
            Some(Instruction::RelativeBaseAdjust { m1 }) => {
                match self.get(self.instruction_pointer + 1, &m1) {
                    None => StepResult::Crash,
                    Some(p1) => {
                        self.relative_base += p1;
                        StepResult::Fwd(2)
                    }
                }
            }
        };

        match step_result {
            StepResult::Crash => {
                return State::Crashed;
            },
            StepResult::Output(out) => {
                self.instruction_pointer += 2;
                match self.current_instruction() {
                    Some(Instruction::Input { .. }) => {
                        return State::OutputAwaitingInput(out)
                    },
                    _ => {
                        return State::Output(out)
                    }
                }
            },
            StepResult::Fwd(len) => {
                self.instruction_pointer += len;
            },
            _ => ()
        };
        match self.current_instruction() {
            Some(Instruction::Halt) => State::Done,
            Some(Instruction::Input { .. }) if self.input_buffer.is_empty() => State::AwaitingInput,
            None => State::Crashed,
            _ => State::Running
        }
    }
    
    pub fn run_and_print(&mut self, inputs: &[i64]) -> Option<i64> {
        self.run(inputs, |x| {println!("Output: {}", &x)})
    }

    pub fn run<F>(&mut self, inputs: &[i64], mut on_output: F) -> Option<i64>
    where F: FnMut(i64) {
        for input in inputs {
            self.read_input(*input);
        }
        loop {
            let state = self.await_output();
            match state {
                State::Output(out) => {
                    on_output(out);
                    continue
                },
                State::Done => return self.return_code,
                State::Crashed => {
                    eprintln!("Program reports crashed state");
                    return self.return_code
                },
                State::AwaitingInput if self.input_buffer.is_empty() => {
                    eprintln!("Program wants input but none available");
                    return self.return_code
                },
                State::AwaitingInput => continue,
                State::OutputAwaitingInput(out) if self.input_buffer.is_empty() => {
                    on_output(out);
                    eprintln!("Program wants input but none available");
                    return self.return_code
                },
                State::OutputAwaitingInput(out) => {
                    on_output(out);
                    continue
                },
                State::Running => continue
            }
        }
    }

    pub fn await_output(&mut self) -> State {
        match self.current_instruction() {
            None => State::Crashed,
            Some(Instruction::Input { .. }) if self.input_buffer.is_empty() => State::AwaitingInput,
            _ => {
                loop {
                    match self.step() {
                        State::Running => continue,
                        state => return state
                    }
                }
            }
        }
    }

    pub fn is_terminated(&self) -> bool {
        match self.current_instruction() {
            Some(Instruction::Halt) => true,
            _ => false
        }
    }

    pub fn overwrite_memory(&mut self, idx: usize, word: i64) {
        self.memory[idx] = word;
    }
}

#[derive(PartialEq, Debug)]
pub enum State {
    Output(i64),
    OutputAwaitingInput(i64),
    AwaitingInput,
    Running,
    Done,
    Crashed
}


enum Instruction {
    Halt,
    Add { m1: ParameterMode, m2: ParameterMode, m3: ParameterMode },
    Mult { m1: ParameterMode, m2: ParameterMode, m3: ParameterMode },
    Input { m1: ParameterMode },
    Output { m1: ParameterMode },
    JumpIfTrue { m1: ParameterMode, m2: ParameterMode },
    JumpIfFalse { m1: ParameterMode, m2: ParameterMode },
    LessThan { m1: ParameterMode, m2: ParameterMode, m3: ParameterMode },
    Equals { m1: ParameterMode, m2: ParameterMode, m3: ParameterMode },
    RelativeBaseAdjust { m1: ParameterMode }
}

impl Instruction {
    fn parse(abcde: &i64) -> Option<Instruction> {
        match abcde.rem_euclid(100) {
            99 => Some(Instruction::Halt),
            1 => {
                Instruction::parse_three(abcde / 100).map(|(m1, m2, m3)| {
                    Instruction::Add { m1, m2, m3 }
                })
            },
            2 => {
                Instruction::parse_three(abcde / 100).map(|(m1, m2, m3)| {
                    Instruction::Mult { m1, m2, m3 }
                })
            },
            3 => {
                Instruction::parse_one(abcde / 100).map(|m1| Instruction::Input { m1 })
            },
            4 => {
                Instruction::parse_one(abcde / 100).map(|m1| Instruction::Output { m1 })
            },
            5 => {
                Instruction::parse_two(abcde / 100).map(|(m1, m2)| {
                    Instruction::JumpIfTrue { m1, m2 }
                })
            },
            6 => {
                Instruction::parse_two(abcde / 100).map(|(m1, m2)| {
                    Instruction::JumpIfFalse { m1, m2 }
                })
            },
            7 => {
                Instruction::parse_three(abcde / 100).map(|(m1, m2, m3)| {
                    Instruction::LessThan { m1, m2, m3 }
                })
            },
            8 => {
                Instruction::parse_three(abcde / 100).map(|(m1, m2, m3)| {
                    Instruction::Equals { m1, m2, m3 }
                })
            },
            9 => {
                Instruction::parse_one(abcde / 100).map(|m1| Instruction::RelativeBaseAdjust { m1 })
            }
            _ => None
        }
    }

    fn parse_one(abc: i64) -> Option<ParameterMode> {
        ParameterMode::of(&abc.rem_euclid(10))
    }

    fn parse_two(abc: i64) -> Option<(ParameterMode, ParameterMode)> {
        let c = abc.rem_euclid(10);
        let b = (abc / 10).rem_euclid(10);
        ParameterMode::of(&c).and_then(|m1| {
            ParameterMode::of(&b).map(|m2| {
                (m1, m2)
            })
        })
    }

    fn parse_three(abc: i64) -> Option<(ParameterMode, ParameterMode, ParameterMode)> {
        let c = abc.rem_euclid(10);
        let b = (abc / 10).rem_euclid(10);
        let a = (abc / 100).rem_euclid(10);
        ParameterMode::of(&c).and_then(|m1| {
            ParameterMode::of(&b).and_then(|m2| {
                ParameterMode::of(&a).map(|m3| {
                    (m1, m2, m3)
                })
            })
        })
    }
}

enum ParameterMode {
    Positional,
    Immediate,
    Relative
}

impl ParameterMode {
    fn of(k: &i64) -> Option<ParameterMode> {
        match k {
            0 => Some(ParameterMode::Positional),
            1 => Some(ParameterMode::Immediate),
            2 => Some(ParameterMode::Relative),
            _ => None            
        }
    }
}

#[cfg(test)]
mod day02_tests {
    use super::*;

    #[test]
    fn add_spec() {
        let mut program = Program::from_str("1,0,0,0,99");
        assert_eq!(program.step(), State::Done);
        assert_eq!(program.memory[..], [2,0,0,0,99]);
        assert_eq!(program.instruction_pointer, 4);

        assert_eq!(program.step(), State::Done);
    }

    #[test]
    fn multiply_spec() {
        let mut program = Program::from_str("2,3,0,3,99");

        assert_eq!(program.step(), State::Done);
        assert_eq!(program.memory[..], [2,3,0,6,99]);
        assert_eq!(program.instruction_pointer, 4);

        assert_eq!(program.step(), State::Done);

        let mut program = Program::from_str("2,4,4,5,99,0");
        assert_eq!(program.step(), State::Done);
        assert_eq!(program.memory[..], [2,4,4,5,99,9801]);
        assert_eq!(program.instruction_pointer, 4);

        assert_eq!(program.step(), State::Done);
    }
}

#[cfg(test)]
mod day05_tests {
    use super::*;

    #[test]
    fn equal_test() {
        let mut program = Program::from_str("3,9,8,9,10,9,4,9,99,-1,8");

        program.read_input(8);
        assert_eq!(program.step(), State::Running);

        assert_eq!(program.step(), State::Running);

        assert_eq!(program.step(), State::Output(1));

        let mut program = Program::from_str("3,9,8,9,10,9,4,9,99,-1,8");
        program.read_input(17);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(0));

        let mut program = Program::from_str("3,3,1108,-1,8,3,4,3,99");
        program.read_input(8);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(1));

        let mut program = Program::from_str("3,3,1108,-1,8,3,4,3,99");
        program.read_input(-17);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(0));
    }

    #[test]
    fn less_than_test() {
        let code = "3,9,7,9,10,9,4,9,99,-1,8";
        let mut program = Program::from_str(code);
        program.read_input(-17);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(1));

        let mut program = Program::from_str(code);
        program.read_input(8);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(0));

        let mut program = Program::from_str(code);
        program.read_input(31);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(0));

        let code = "3,3,1107,-1,8,3,4,3,99";
        let mut program = Program::from_str(code);
        program.read_input(-17);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(1));

        let mut program = Program::from_str(code);
        program.read_input(8);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(0));

        let mut program = Program::from_str(code);
        program.read_input(31);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.step(), State::Output(0));
    }

    #[test]
    fn jump_test() {
        let code = "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9";
        let mut program = Program::from_str(code);
        program.read_input(0);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 2);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 9);

        assert_eq!(program.step(), State::Output(0));

        let mut program = Program::from_str(code);
        program.read_input(-17);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 2);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 5);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 9);

        assert_eq!(program.step(), State::Output(1));
        assert_eq!(program.instruction_pointer, 11);

        let mut program = Program::from_str(code);
        program.read_input(42);
        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 2);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 5);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 9);

        assert_eq!(program.step(), State::Output(1));
        assert_eq!(program.instruction_pointer, 11);
    }

    #[test]
    fn jump_test_2() {
        let code = "3,3,1105,-1,9,1101,0,0,12,4,12,99,1";
        let mut program = Program::from_str(code);
        let input = 0;
        program.read_input(input);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 2);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 5);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 9);

        assert_eq!(program.step(), State::Output((input != 0) as i64));

        let mut program = Program::from_str(code);
        let input = 17;
        program.read_input(input);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 2);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 9);

        assert_eq!(program.step(), State::Output((input != 0) as i64));

        let mut program = Program::from_str(code);
        let input = -256;
        program.read_input(input);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 2);

        assert_eq!(program.step(), State::Running);
        assert_eq!(program.instruction_pointer, 9);

        assert_eq!(program.step(), State::Output((input != 0) as i64));
    }

    #[test]
    fn longer_test() {
        let code = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";

        let mut program = Program::from_str(code);
        program.read_input(-3);
        loop {
            match program.step() {
                State::Crashed | State::Done => panic!(),
                State::Output(x) => {
                    assert_eq!(x, 999);
                    break
                },
                _ => () // continue
            }
        }

        let mut program = Program::from_str(code);
        program.read_input(8);

        loop {
            match program.step() {
                State::Crashed | State::Done => panic!(),
                State::Output(x) => {
                    assert_eq!(x, 1000);
                    break
                },
                _ => () // continue
            }
        }

        let mut program = Program::from_str(code);
        program.read_input(88);

        loop {
            match program.step() {
                State::Crashed | State::Done => panic!(),
                State::Output(x) => {
                    assert_eq!(x, 1001);
                    break
                },
                _ => () // continue
            }
        }
    }

    #[test]
    fn clone_test() {
        let code = "3,3,1105,-1,9,1101,0,0,12,4,12,99,1";
        let mut program = Program::from_str(code);
        let program2 = program.clone();

        program.read_input(-1);
        program.step();

        assert_ne!(program.instruction_pointer, program2.instruction_pointer);
    }
}

#[cfg(test)]
mod relative_base_test {
    use super::*;

    #[test]
    fn quine_test() {
        let mut program = Program::from_str("109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99");
        let mut outputs = vec!();
        program.run(&[], &mut |x| { outputs.push(x)});

        assert_eq!(outputs[..], [109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99]);
    }

    #[test]
    fn long_test() {
        let mut program = Program::from_str("1102,34915192,34915192,7,4,7,99,0");
        let mut out = 0;
        program.run(&[], &mut |x| { out = x});

        let out_str = format!("{}", out);
        assert_eq!(out_str.len(), 16);

        let mut program = Program::from_str("104,1125899906842624,99");
        out = 0;
        program.run(&[], &mut |x| { out = x});
        
        assert_eq!(out, 1125899906842624);
    }
}