use fox_bytecode::{BackendService, ErrorInfo, InterpreterService, probe::ProbeBackendService};

#[derive(Default)]
pub struct Probe {
    compiler_errors: Vec<ErrorInfo>,
    backend: ProbeBackendService,
}

impl Probe {
    pub fn stack_trace_text(&self) -> Option<String> {
        let Some(trace) = &self.backend.stack_trace else {
            return None;
        };
        let val = trace
            .iter()
            .map(|elem| format!("{elem}"))
            .collect::<Vec<_>>()
            .join("\n");
        Some(val)
    }

    pub fn top_error_message(&self) -> Option<&str> {
        if let Some(err) = self.compiler_errors.first() {
            return Some(err.message());
        }
        if let Some(err) = &self.backend.error {
            return Some(err.message());
        }
        None
    }

    pub fn assert_output_match<T: AsRef<str>>(&self, output: &[T]) {
        self.backend.assert_output_match(output)
    }
}

impl BackendService for Probe {
    fn print_value(&mut self, value: fox_bytecode::Value) {
        self.backend.print_value(value);
    }

    fn set_error(&mut self, error: fox_bytecode::MachineError) {
        self.backend.set_error(error);
    }

    fn set_stack_trace(&mut self, stack_trace: Vec<fox_bytecode::StackTraceElement>) {
        self.backend.set_stack_trace(stack_trace);
    }
}

impl InterpreterService for Probe {
    fn set_compile_errors(&mut self, errors: &[ErrorInfo]) {
        self.compiler_errors = errors.to_vec();
    }
}
