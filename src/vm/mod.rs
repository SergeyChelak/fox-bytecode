mod instruction;
pub use instruction::*;

mod machine;
pub use machine::Machine;

use crate::{data::DataType, vm::machine::MachineError};

pub trait MachineIO {
    fn push_output(&mut self, value: DataType);

    fn push_error(&mut self, error: MachineError);
}

pub struct SystemIO;

impl MachineIO for SystemIO {
    fn push_output(&mut self, value: DataType) {
        println!("{value}");
    }

    fn push_error(&mut self, _error: MachineError) {
        // no op
    }
}

#[cfg(test)]
pub mod probe {
    use super::{DataType, MachineError, MachineIO};

    pub struct Probe {
        output_buffer: Vec<String>,
        error: Option<MachineError>,
    }

    impl Probe {
        pub fn new() -> Self {
            Self {
                output_buffer: Vec::new(),
                error: None,
            }
        }

        pub fn is_output_matches(&self, output: &[String]) -> bool {
            if self.output_buffer.len() != output.len() {
                return false;
            }
            self.output_buffer
                .iter()
                .zip(output.iter())
                .all(|(l, r)| l == r)
        }

        pub fn output_to_string(&self) -> String {
            self.output_buffer.join("\n")
        }
    }

    impl MachineIO for Probe {
        fn push_output(&mut self, value: DataType) {
            self.output_buffer.push(value.to_string());
        }

        fn push_error(&mut self, error: MachineError) {
            self.error = Some(error);
        }
    }
}
