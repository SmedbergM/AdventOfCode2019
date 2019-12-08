use std::io;
use io::prelude::*;
use intcode::Program;

fn read_one_line_from_stdin() -> String {
    let stdin = io::stdin();
    let line = stdin.lock().lines().next().unwrap().unwrap();
    line
}

fn main() {
    let line = read_one_line_from_stdin();
    let mut program = Program::from_str(&line);
    let return_code = program.run_and_print(&[1]);
    println!("Program (input=1) returned diagnostic code {}", return_code.unwrap());
    let mut program = Program::from_str(&line);
    let return_code = program.run_and_print(&[5]);
    println!("Program (input=5) returned diagnostic code {}", return_code.unwrap());
}

