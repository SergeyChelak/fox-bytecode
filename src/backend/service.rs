use crate::{MachineError, StackTraceElement, Value};

pub trait BackendService {
    fn print_value(&mut self, value: Value);

    fn set_error(&mut self, error: MachineError);

    fn set_stack_trace(&mut self, stack_trace: Vec<StackTraceElement>);
}

pub struct VirtualMachineService;

impl BackendService for VirtualMachineService {
    fn print_value(&mut self, value: Value) {
        println!("{value}");
    }

    fn set_error(&mut self, error: MachineError) {
        eprintln!("Runtime error: {error}")
    }

    fn set_stack_trace(&mut self, stack_trace: Vec<StackTraceElement>) {
        eprintln!("Trace:");
        stack_trace.iter().for_each(|elem| eprintln!("> {elem}"));
    }
}

pub mod probe {
    use super::*;

    #[derive(Default)]
    pub struct ProbeBackendService {
        pub print_buffer: Vec<String>,
        pub error: Option<MachineError>,
        pub stack_trace: Option<Vec<StackTraceElement>>,
    }

    impl ProbeBackendService {
        pub fn assert_output_match<T: AsRef<str>>(&self, output: &[T]) {
            for (l, r) in self.print_buffer.iter().zip(output.iter()) {
                assert_eq!(l, r.as_ref())
            }
            assert_eq!(
                self.print_buffer.len(),
                output.len(),
                "Output buffer line count mismatch"
            )
        }
    }

    impl BackendService for ProbeBackendService {
        fn print_value(&mut self, value: Value) {
            self.print_buffer.push(value.to_string());
        }

        fn set_error(&mut self, error: MachineError) {
            self.error = Some(error);
        }

        fn set_stack_trace(&mut self, stack_trace: Vec<StackTraceElement>) {
            self.stack_trace = Some(stack_trace);
        }
    }
}
