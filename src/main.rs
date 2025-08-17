use std::{process::exit, rc::Rc};

use crate::{compiler::compile, utils::ErrorFormatter, vm::Machine};

mod chunk;
mod compiler;
mod utils;
mod value;
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
    interpret(code);
}

fn interpret(code: Vec<char>) {
    let code_ref = Rc::new(code);
    let result = compile(code_ref.clone());
    match result {
        Ok(chunk) => {
            let mut vm = Machine::with(chunk);
            let result = vm.run();

            if let Err(err) = result {
                eprintln!("Runtime error: {err}")
            }
        }
        Err(arr) => {
            let formatter = ErrorFormatter::with(code_ref);
            for err in &arr {
                formatter.format_error(err);
            }
        }
    }
}

fn show_usage() {
    println!("Usage: fox-bytecode <script.fox>");
}
