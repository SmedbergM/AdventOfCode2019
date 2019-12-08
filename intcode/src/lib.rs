use std::collections::VecDeque;

#[derive(Clone)]
pub struct Program {
    memory: Vec<i32>,
    instruction_pointer: usize,
    return_code: Option<i32>,
    input_buffer: VecDeque<i32>
}

impl Program {
    pub fn from_str(line: &str) -> Program {
        let memory: Vec<i32> = line.split(",")
            .flat_map(|s| i32::from_str_radix(s, 10).ok()).collect();
        Program { memory,
            instruction_pointer: 0,
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

    fn get(&self, idx: usize, mode: &ParameterMode) -> i32 {
        match mode {
            ParameterMode::Immediate => self.memory[idx],
            ParameterMode::Positional => self.memory[self.memory[idx] as usize]
        }
    }

    pub fn read_input(&mut self, input: i32) {
        self.input_buffer.push_back(input)
    }

    fn set(&mut self, idx: usize, value: i32) {
        let idx2 = self.memory[idx] as usize;
        self.memory[idx2] = value;
    }

    fn perform_add(&mut self, m1: &ParameterMode, m2: &ParameterMode) {
        let addend1 = self.get(self.instruction_pointer + 1, m1);
        let addend2 = self.get(self.instruction_pointer + 2, m2);
        self.set(self.instruction_pointer + 3, addend1 + addend2);
    }

    fn perform_mult(&mut self, m1: &ParameterMode, m2: &ParameterMode) {
        let factor1 = self.get(self.instruction_pointer + 1, m1);
        let factor2 = self.get(self.instruction_pointer + 2, m2);
        self.set(self.instruction_pointer + 3, factor1 * factor2);
    }

    fn perform_save(&mut self, input: Option<i32>) {
        self.set(self.instruction_pointer + 1, input.unwrap())
    }

    fn perform_jump_if(&mut self, nonzero: bool, m1: &ParameterMode, m2: &ParameterMode) -> bool {
        let p1 = self.get(self.instruction_pointer + 1, m1);
        if (p1 != 0) == nonzero {
            let p2 = self.get(self.instruction_pointer + 2, m2);
            self.instruction_pointer = p2 as usize;
            true
        } else {
            false
        }
    }

    fn perform_less_than(&mut self, m1: &ParameterMode, m2: &ParameterMode) {
        let p1 = self.get(self.instruction_pointer + 1, m1);
        let p2 = self.get(self.instruction_pointer + 2, m2);
        self.set(self.instruction_pointer + 3, (p1 < p2) as i32);
    }

    fn perform_equals(&mut self, m1: &ParameterMode, m2: &ParameterMode) {
        let p1 = self.get(self.instruction_pointer + 1, m1);
        let p2 = self.get(self.instruction_pointer + 2, m2);
        self.set(self.instruction_pointer + 3, (p1 == p2) as i32);
    }

    fn step<'a, F>(&mut self, on_output: &mut F) -> State
        where F: FnMut(i32) {

            let instruction = self.current_instruction().unwrap();
            let (next_state, jumped) = match &instruction { // in a real implementation, this would probably need to be wrapped in a Result or something
                Instruction::Halt => (State::Done, false),
                Instruction::Add { m1, m2 } => {
                    self.perform_add(&m1, &m2);
                    (State::Running, false)
                },
                Instruction::Mult { m1, m2 } => {
                    self.perform_mult(&m1, &m2);
                    (State::Running, false)
                },
                Instruction::Save => {
                    let next_input = self.input_buffer.pop_front();
                    self.perform_save(next_input);
                    (State::Running, false)
                },
                Instruction::Output { m1 } => {
                    let output = self.get(self.instruction_pointer + 1, &m1);
                    on_output(output);
                    self.return_code = Some(output);
                    (State::Running, false)
                },
                Instruction::JumpIfTrue { m1, m2 } => {
                    let jumped = self.perform_jump_if(true, &m1, &m2);
                    (State::Running, jumped)
                },
                Instruction::JumpIfFalse { m1, m2 } => {
                    let jumped = self.perform_jump_if(false, &m1, &m2);
                    (State::Running, jumped)
                },
                Instruction::LessThan { m1, m2 } => {
                    self.perform_less_than(&m1, &m2);
                    (State::Running, false)
                },
                Instruction::Equals { m1, m2 } => {
                    self.perform_equals(&m1, &m2);
                    (State::Running, false)
                }
            };
            self.instruction_pointer += instruction.len(jumped);
            next_state
        }    

    pub fn run_and_print(&mut self, inputs: &[i32]) -> Option<i32> {
        self.run(inputs, |x| {println!("Output: {}", &x)})
    }

    pub fn run<F>(&mut self, inputs: &[i32], mut on_output: F) -> Option<i32>
        where F: FnMut(i32) {
        for input in inputs {
            self.read_input(*input);
        }
        while let State::Running = self.step(&mut on_output) {};
        self.return_code
    }

    pub fn await_output<F>(&mut self, on_output: &mut F) -> Option<i32>
    where F: FnMut(i32) {
        
        let mut last_output = None;
        while let None = last_output {
            if let State::Done = self.step(&mut |x| {
                last_output = Some(x);
                on_output(x);
            }) {
                break
            }
        };
        return last_output
    }

    pub fn is_terminated(&self) -> bool {
        match self.current_instruction() {
            Some(Instruction::Halt) => true,
            _ => false
        }
    }
}

#[derive(PartialEq, Debug)]
enum State {
    Running,
    Done
}


enum Instruction {
    Halt,
    Add { m1: ParameterMode, m2: ParameterMode },
    Mult { m1: ParameterMode, m2: ParameterMode },
    Save,
    Output { m1: ParameterMode },
    JumpIfTrue { m1: ParameterMode, m2: ParameterMode },
    JumpIfFalse { m1: ParameterMode, m2: ParameterMode },
    LessThan { m1: ParameterMode, m2: ParameterMode },
    Equals { m1: ParameterMode, m2: ParameterMode }
}

impl Instruction {
    fn len(&self, jump: bool) -> usize {
        match self {
            _ if jump => 0,
            Instruction::Halt => 1,
            Instruction::Add { .. } => 4,
            Instruction::Mult { .. } => 4,
            Instruction::Save => 2,
            Instruction::Output { .. } => 2,
            Instruction::JumpIfTrue { .. } => 3,
            Instruction::JumpIfFalse { .. } => 3,
            Instruction::LessThan { .. } => 4,
            Instruction::Equals { .. } => 4
        }
    }

    fn parse(abcde: &i32) -> Option<Instruction> {
        match abcde.rem_euclid(100) {
            99 => Some(Instruction::Halt),
            1 => Instruction::parse_add(abcde / 100),
            2 => Instruction::parse_mult(abcde / 100),
            3 => Some(Instruction::Save),
            4 => Instruction::parse_output(abcde / 100),
            5 => Instruction::parse_jump_if_true(abcde / 100),
            6 => Instruction::parse_jump_if_false(abcde / 100),
            7 => Instruction::parse_less_than(abcde / 100),
            8 => Instruction::parse_equals(abcde / 100),
            _ => None
        }
    }

    fn parse_add(abc: i32) -> Option<Instruction> {
        let c = abc.rem_euclid(10);
        let b = (abc / 10).rem_euclid(10);
        ParameterMode::of(&c).and_then(|m1| {
            ParameterMode::of(&b).map(|m2| {
                Instruction::Add { m1, m2 }
            })
        })
    }

    fn parse_mult(abc: i32) -> Option<Instruction> {
        let c = abc.rem_euclid(10);
        let b = (abc / 10).rem_euclid(10);
        ParameterMode::of(&c).and_then(|m1| {
            ParameterMode::of(&b).map(|m2| {
                Instruction::Mult { m1, m2 }
            })
        })  
    }

    fn parse_output(abc: i32) -> Option<Instruction> {
        let c = abc.rem_euclid(10);
        ParameterMode::of(&c).map(|m1| Instruction::Output { m1 })
    }

    fn parse_jump_if_true(abc: i32) -> Option<Instruction> {
        let c = abc.rem_euclid(10);
        let d = (abc / 10).rem_euclid(10);
        ParameterMode::of(&c).and_then(|m1| { ParameterMode::of(&d).map(|m2| {
            Instruction::JumpIfTrue { m1, m2 }
        })})
    }

    fn parse_jump_if_false(abc: i32) -> Option<Instruction> {
        let c = abc.rem_euclid(10);
        let d = (abc / 10).rem_euclid(10);
        ParameterMode::of(&c).and_then(|m1| { ParameterMode::of(&d).map(|m2| {
            Instruction::JumpIfFalse { m1, m2 }
        })})
    }

    fn parse_less_than(abc: i32) -> Option<Instruction> {
        let c = abc.rem_euclid(10);
        let d = (abc / 10).rem_euclid(10);
        ParameterMode::of(&c).and_then(|m1| { ParameterMode::of(&d).map(|m2| {
            Instruction::LessThan { m1, m2 }
        })})
    }

    fn parse_equals(abc: i32) -> Option<Instruction> {
        let c = abc.rem_euclid(10);
        let d = (abc / 10).rem_euclid(10);
        ParameterMode::of(&c).and_then(|m1| { ParameterMode::of(&d).map(|m2| {
            Instruction::Equals { m1, m2 }
        })})
    }
}

enum ParameterMode {
    Positional,
    Immediate
}

impl ParameterMode {
    fn of(k: &i32) -> Option<ParameterMode> {
        match k {
            0 => Some(ParameterMode::Positional),
            1 => Some(ParameterMode::Immediate),
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
        let mut output_count = 0;
        assert_eq!(program.step(&mut |_| { output_count += 1}), State::Running);
        assert_eq!(program.memory[..], [2,0,0,0,99]);
        assert_eq!(program.instruction_pointer, 4);

        assert_eq!(program.step(&mut |_| { output_count += 1}), State::Done);
        assert_eq!(output_count, 0);
    }

    #[test]
    fn multiply_spec() {
        let mut program = Program::from_str("2,3,0,3,99");
        let mut output_count = 0;

        assert_eq!(program.step(&mut |_| { output_count += 1}), State::Running);
        assert_eq!(program.memory[..], [2,3,0,6,99]);
        assert_eq!(program.instruction_pointer, 4);

        assert_eq!(program.step(&mut |_| { output_count += 1}), State::Done);
        assert_eq!(output_count, 0);

        let mut program = Program::from_str("2,4,4,5,99,0");
        assert_eq!(program.step(&mut |_| { output_count += 1}), State::Running);
        assert_eq!(program.memory[..], [2,4,4,5,99,9801]);
        assert_eq!(program.instruction_pointer, 4);

        assert_eq!(program.step(&mut |_| { output_count += 1}), State::Done);
    }
}

#[cfg(test)]
mod day05_tests {
    use super::*;

    #[test]
    fn equal_test() {
        let mut program = Program::from_str("3,9,8,9,10,9,4,9,99,-1,8");
        let mut last_output = None;

        program.read_input(8);
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(1));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str("3,9,8,9,10,9,4,9,99,-1,8");
        program.read_input(17);
        last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(0));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str("3,3,1108,-1,8,3,4,3,99");
        program.read_input(8);
        last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(1));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str("3,3,1108,-1,8,3,4,3,99");
        program.read_input(-17);
        last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(0));
        assert_eq!(state, State::Running);
    }

    #[test]
    fn less_than_test() {
        let code = "3,9,7,9,10,9,4,9,99,-1,8";
        let mut program = Program::from_str(code);
        program.read_input(-17);

        let mut last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(1));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str(code);
        program.read_input(8);
        last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(0));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str(code);
        program.read_input(31);
        last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(0));
        assert_eq!(state, State::Running);

        let code = "3,3,1107,-1,8,3,4,3,99";
        let mut program = Program::from_str(code);
        program.read_input(-17);
        last_output = None;

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, None);
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(1));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str(code);
        program.read_input(8);
        last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(0));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str(code);
        program.read_input(31);
        last_output = None;

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(0));
        assert_eq!(state, State::Running);
    }

    #[test]
    fn jump_test() {
        let code = "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9";
        let mut program = Program::from_str(code);
        let mut last_output = None;
        program.read_input(0);
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 2);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 9);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some(0));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str(code);
        program.read_input(-17);
        last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 2);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 5);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 9);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(last_output, Some(1));
        assert_eq!(program.instruction_pointer, 11);

        let mut program = Program::from_str(code);
        program.read_input(42);
        last_output = None;
        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 2);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 5);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 9);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(last_output, Some(1));
        assert_eq!(program.instruction_pointer, 11);
    }

    #[test]
    fn jump_test_2() {
        let code = "3,3,1105,-1,9,1101,0,0,12,4,12,99,1";
        let mut program = Program::from_str(code);
        let input = 0;
        program.read_input(input);
        let mut last_output = None;

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 2);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 5);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 9);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some((input != 0) as i32));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str(code);
        let input = 17;
        program.read_input(input);
        last_output = None;

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 2);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 9);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some((input != 0) as i32));
        assert_eq!(state, State::Running);

        let mut program = Program::from_str(code);
        let input = -256;
        program.read_input(input);
        last_output = None;

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 2);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(state, State::Running);
        assert_eq!(program.instruction_pointer, 9);
        assert_eq!(last_output, None);

        let state = program.step(&mut |x| {last_output = Some(x)});
        assert_eq!(last_output, Some((input != 0) as i32));
        assert_eq!(state, State::Running);
    }

    #[test]
    fn longer_test() {
        let code = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";

        let mut last_output = None;

        let mut program = Program::from_str(code);
        program.read_input(-3);
        while let None = last_output {
            program.step(&mut |x| {last_output = Some(x)});
        }
        assert_eq!(last_output, Some(999));

        let mut program = Program::from_str(code);
        program.read_input(8);
        last_output = None;

        while let None = last_output {
            program.step(&mut |x| {last_output = Some(x)});
        }
        assert_eq!(last_output, Some(1000));


        let mut program = Program::from_str(code);
        program.read_input(88);
        last_output = None;

        while let None = last_output {
            program.step(&mut |x| {last_output = Some(x)});
        }
        assert_eq!(last_output, Some(1001));
    }

    #[test]
    fn clone_test() {
        let code = "3,3,1105,-1,9,1101,0,0,12,4,12,99,1";
        let mut program = Program::from_str(code);
        let program2 = program.clone();

        program.read_input(-1);
        program.step(&mut |_| { });

        assert_ne!(program.instruction_pointer, program2.instruction_pointer);
    }
}