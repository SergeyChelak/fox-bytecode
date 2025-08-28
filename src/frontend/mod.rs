mod assembler;
mod compiler;
mod rule;
mod scanner;
mod token;

use std::rc::Rc;

use scanner::*;
pub use token::*;

use crate::{Closure, errors::ErrorInfo, frontend::assembler::Assembler};

pub fn compile(code: Rc<Vec<char>>) -> Result<Closure, Vec<ErrorInfo>> {
    let scanner = Scanner::new(code);
    let frontend = Assembler::new(Box::new(scanner));
    let func = frontend.compile()?;
    let closure = Closure::with(func);
    Ok(closure)
}
