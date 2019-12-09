use intcode::Program;

fn main() {
    let line = util::read_single_line_from_stdin().unwrap();
    let program = Program::from_str(&line);
    program.clone().run_and_print(&[1]);

    println!("Locking on to Ceres...");
    program.clone().run_and_print(&[2]);
}
