use std::{cell::RefCell, rc::Rc};

mod data;
pub use data::*;
mod frontend;
use frontend::*;
mod errors;
pub use errors::*;
mod utils;
pub use utils::file_to_chars;
mod backend;
pub use backend::*;

pub fn interpret(code_ref: Rc<Vec<char>>, io: Rc<RefCell<dyn MachineIO>>) {
    let result = compile(code_ref.clone());
    match result {
        Ok(chunk) => {
            let mut vm = Machine::with(chunk, io.clone());
            let result = vm.run();

            if result.is_err() {
                io.borrow_mut().push_output(Value::text_from_str(
                    "Completed with errors. See messages above",
                ));
            }
        }
        Err(arr) => {
            io.borrow_mut().set_scanner_errors(&arr);
        }
    }
}
