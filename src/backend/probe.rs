use crate::{ErrorInfo, MachineError, MachineIO, StackTraceElement, data::Value};

#[derive(Default)]
pub struct Probe {
    output_buffer: Vec<String>,
    vm_error: Option<MachineError>,
    scanner_errors: Vec<ErrorInfo>,
    stack_trace: Option<Vec<StackTraceElement>>,
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

    pub fn top_error_message(&self) -> Option<&str> {
        if let Some(err) = self.scanner_errors.first() {
            return Some(err.message());
        }
        if let Some(err) = &self.vm_error {
            return Some(err.message());
        }
        None
    }

    pub fn stack_trace_text(&self) -> Option<String> {
        let Some(trace) = &self.stack_trace else {
            return None;
        };
        let val = trace
            .iter()
            .map(|elem| format!("{elem}"))
            .collect::<Vec<_>>()
            .join("\n");
        Some(val)
    }
}

impl MachineIO for Probe {
    fn push_output(&mut self, value: Value) {
        self.output_buffer.push(value.to_string());
    }

    fn set_vm_error(&mut self, error: MachineError) {
        self.vm_error = Some(error);
    }

    fn set_scanner_errors(&mut self, errors: &[ErrorInfo]) {
        self.scanner_errors = errors.to_vec();
    }

    fn set_stack_trace(&mut self, stack_trace: Vec<StackTraceElement>) {
        self.stack_trace = Some(stack_trace);
    }
}
