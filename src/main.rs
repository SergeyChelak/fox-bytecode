use std::{cell::RefCell, process::exit, rc::Rc};

use fox_bytecode::*;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        2 => run_file(&args[1]),
        _ => show_usage(),
    }
}

fn run_file<T: AsRef<str>>(path: T) {
    let Ok(code) = file_to_chars(&path) else {
        eprintln!("Failed to open file {}", path.as_ref());
        exit(-1);
    };
    let code_ref = Rc::new(code);
    let formatter = ErrorFormatter::with(code_ref.clone());
    let machine_io = SystemIO::new(formatter);
    interpret(code_ref, Rc::new(RefCell::new(machine_io)));
}

fn show_usage() {
    println!("Usage: fox-bytecode <script.fox>");
}
