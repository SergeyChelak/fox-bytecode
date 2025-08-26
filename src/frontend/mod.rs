mod assembler;
mod compiler;
mod rule;
mod scanner;
mod token;

use std::rc::Rc;

use scanner::*;
pub use token::*;

use crate::{Func, errors::ErrorInfo, frontend::assembler::Assembler};

pub fn compile(code: Rc<Vec<char>>) -> Result<Func, Vec<ErrorInfo>> {
    let scanner = Scanner::new(code);
    let frontend = Assembler::new(Box::new(scanner));
    let compiler = frontend.compile()?;
    let func = compiler.function();
    Ok(func)
}
