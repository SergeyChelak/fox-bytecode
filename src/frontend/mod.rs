mod compiler;
mod core;
mod rule;
mod scanner;
mod token;

use std::rc::Rc;

use scanner::*;
pub use token::*;

use crate::{Chunk, Func, errors::ErrorInfo, frontend::core::Frontend};

pub fn _compile(code: Rc<Vec<char>>) -> Result<Func, Vec<ErrorInfo>> {
    let scanner = Scanner::new(code);
    let frontend = Frontend::new(Box::new(scanner));
    let compiler = frontend.compile()?;
    let func = compiler.function();
    Ok(func)
}

pub fn compile(code: Rc<Vec<char>>) -> Result<Chunk, Vec<ErrorInfo>> {
    let func = _compile(code)?;
    Ok(func.chunk().clone())
}
