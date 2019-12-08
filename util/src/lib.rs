use std::io;
use std::io::prelude::*;

pub fn read_single_line_from_stdin() -> Option<String> {
    let stdin = io::stdin();
    let opt_line = stdin.lock().lines().next();
    opt_line.and_then(|a| a.ok())
}