use std::fmt::Display;

mod call_frame;
mod machine;
pub use machine::Machine;
mod service;
pub use service::*;

use crate::NativeFn;

pub trait NativeFunctionsProvider {
    fn get_functions(&self) -> Vec<(String, NativeFn)>;
}

pub struct EmptyNative;

impl NativeFunctionsProvider for EmptyNative {
    fn get_functions(&self) -> Vec<(String, NativeFn)> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct MachineError {
    text: String,
    line_number: Option<usize>,
}

impl MachineError {
    pub fn with_str(msg: &str) -> Self {
        Self {
            text: msg.to_string(),
            line_number: None,
        }
    }

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

pub struct StackTraceElement {
    pub line: Option<usize>,
    pub func_name: Option<String>,
}

impl Display for StackTraceElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.func_name.as_deref().unwrap_or("script");
        let line = self
            .line
            .as_ref()
            .map(|s| format!("{s}"))
            .unwrap_or("???".to_string());
        write!(f, "[line {line}] in {name}")
    }
}
