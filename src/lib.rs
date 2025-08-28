use std::rc::Rc;

mod data;
pub use data::*;
mod frontend;
use frontend::*;
mod errors;
pub use errors::*;
mod utils;
pub use utils::*;
mod backend;
pub use backend::*;

pub fn interpret(
    code_ref: Rc<Vec<char>>,
    interpreter_service: Shared<dyn InterpreterService>,
    backend_service: Shared<dyn BackendService>,
) {
    let result = compile(code_ref.clone());
    match result {
        Ok(chunk) => {
            let native_fn_provider = ProductionNativeFunctions;
            let mut vm = Machine::with(chunk, backend_service.clone(), native_fn_provider);
            let result = vm.run();

            if result.is_err() {
                backend_service
                    .borrow_mut()
                    .print_value(Value::text_from_str(
                        "Completed with errors. See messages above",
                    ));
            }
        }
        Err(arr) => {
            interpreter_service.borrow_mut().set_compile_errors(&arr);
        }
    }
}

pub trait InterpreterService {
    fn set_compile_errors(&mut self, errors: &[ErrorInfo]);
}
