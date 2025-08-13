use std::process::exit;

use crate::{scanner::Scanner, vm::MachineError};

mod chunk;
mod scanner;
mod token;
mod vm;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        2 => run_file(&args[1]),
        _ => show_usage(),
    }
}

fn run_file<T: AsRef<str>>(path: T) {
    let p = path.as_ref();
    let Ok(data) = std::fs::read_to_string(p) else {
        eprintln!("Failed to open file {}", p);
        exit(-1);
    };
    let code = data.chars().collect::<Vec<_>>();
    run(code);
}

fn run(code: Vec<char>) {
    // compile
    let mut scanner = Scanner::new(code);
}

fn show_usage() {
    println!("Usage: fox-bytecode <script.fox>");
}

fn show_machine_error(err: MachineError) {
    match err {
        MachineError::Compile(s) | MachineError::Runtime(s) => eprintln!("{s}"),
    }
}
