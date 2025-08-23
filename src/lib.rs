use std::{cell::RefCell, rc::Rc};

mod chunk;
mod compiler;
use compiler::*;
mod data;
pub use data::DataType;
mod error_info;
pub use error_info::*;
mod utils;
pub use utils::file_to_chars;
mod vm;
pub use vm::*;

pub fn interpret(code_ref: Rc<Vec<char>>, io: Rc<RefCell<dyn MachineIO>>) {
    let result = compile(code_ref.clone());
    match result {
        Ok(chunk) => {
            let mut vm = Machine::with(chunk, io.clone());
            let result = vm.run();

            if let Err(err) = result {
                io.borrow_mut().set_vm_error(err);
            }
        }
        Err(arr) => {
            io.borrow_mut().set_scanner_errors(&arr);
        }
    }
}
