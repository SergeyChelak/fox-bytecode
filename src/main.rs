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

pub struct SystemIO {
    formatter: ErrorFormatter,
}

impl SystemIO {
    pub fn new(formatter: ErrorFormatter) -> Self {
        Self { formatter }
    }
}

impl MachineIO for SystemIO {
    fn push_output(&mut self, value: Value) {
        println!("{value}");
    }

    fn set_vm_error(&mut self, error: MachineError) {
        eprintln!("Runtime error: {error}")
    }

    fn set_scanner_errors(&mut self, errors: &[ErrorInfo]) {
        for err in errors {
            let text = self.formatter.format_error(err);
            eprintln!("{text}");
        }
    }

    fn set_stack_trace(&mut self, stack_trace: Vec<StackTraceElement>) {
        eprintln!("Trace:");
        stack_trace.iter().for_each(|elem| eprintln!("> {elem}"));
    }
}
