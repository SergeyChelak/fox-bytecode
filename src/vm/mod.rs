mod instruction;

pub use instruction::*;

mod machine;
pub use machine::Machine;

use crate::{
    data::DataType,
    error_info::{ErrorFormatter, ErrorInfo},
    vm::machine::MachineError,
};

pub trait MachineIO {
    fn push_output(&mut self, value: DataType);

    fn set_vm_error(&mut self, error: MachineError);

    fn set_scanner_errors(&mut self, errors: &[ErrorInfo]);
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
    fn push_output(&mut self, value: DataType) {
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
}

#[derive(Default)]
pub struct Probe {
    output_buffer: Vec<String>,
    vm_error: Option<MachineError>,
    scanner_errors: Vec<ErrorInfo>,
}

impl Probe {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assert_output_match<T: AsRef<str>>(&self, output: &[T]) {
        for (l, r) in self.output_buffer.iter().zip(output.iter()) {
            assert_eq!(l, r.as_ref())
        }
        assert_eq!(
            self.output_buffer.len(),
            output.len(),
            "Output buffer line count mismatch"
        )
    }

    pub fn output_to_string(&self) -> String {
        self.output_buffer.join("\n")
    }

    pub fn vm_error(&self) -> Option<MachineError> {
        self.vm_error.clone()
    }

    pub fn has_errors(&self) -> bool {
        !self.scanner_errors.is_empty() || self.vm_error.is_some()
    }

    pub fn top_error_message(&self) -> Option<String> {
        if let Some(err) = self.scanner_errors.first() {
            return Some(err.message().to_string());
        }
        if let Some(err) = &self.vm_error {
            return Some(err.message().to_string());
        }
        None
    }
}

impl MachineIO for Probe {
    fn push_output(&mut self, value: DataType) {
        self.output_buffer.push(value.to_string());
    }

    fn set_vm_error(&mut self, error: MachineError) {
        self.vm_error = Some(error);
    }

    fn set_scanner_errors(&mut self, errors: &[ErrorInfo]) {
        self.scanner_errors = errors.to_vec();
    }
}
