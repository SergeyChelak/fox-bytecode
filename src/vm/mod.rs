use std::fmt::Display;

mod probe;
pub use probe::*;
mod machine;
pub use machine::Machine;

use crate::{data::Value, errors::ErrorInfo};

#[derive(Debug, Clone)]
pub struct MachineError {
    text: String,
    line_number: Option<usize>,
}

impl MachineError {
    pub fn message(&self) -> &str {
        self.text.as_str()
    }
}

impl Display for MachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = if let Some(num) = self.line_number {
            &format!("{num}")
        } else {
            "???"
        };
        write!(f, "[line {val}] {}", self.text)
    }
}

pub type MachineResult<T> = Result<T, MachineError>;

pub trait MachineIO {
    fn push_output(&mut self, value: Value);

    fn set_vm_error(&mut self, error: MachineError);

    fn set_scanner_errors(&mut self, errors: &[ErrorInfo]);
}
