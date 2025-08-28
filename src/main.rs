use std::{process::exit, rc::Rc};

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
    let int_service = RuntimeInterpreterService::new(formatter);
    let be_service = VirtualMachineService;
    interpret(code_ref, shared(int_service), shared(be_service));
}

fn show_usage() {
    println!("Usage: fox-bytecode <script.fox>");
}

struct RuntimeInterpreterService {
    formatter: ErrorFormatter,
}

impl RuntimeInterpreterService {
    pub fn new(formatter: ErrorFormatter) -> Self {
        Self { formatter }
    }
}

impl InterpreterService for RuntimeInterpreterService {
    fn set_compile_errors(&mut self, errors: &[ErrorInfo]) {
        for err in errors {
            let text = self.formatter.format_error(err);
            eprintln!("{text}");
        }
    }
}
