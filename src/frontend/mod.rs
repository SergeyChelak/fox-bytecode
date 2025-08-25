mod compiler;
mod core;
mod rule;
mod scanner;
mod token;

use std::rc::Rc;

use scanner::*;
pub use token::*;

use crate::{data::Chunk, errors::ErrorInfo, frontend::core::Frontend};

pub fn compile(code: Rc<Vec<char>>) -> Result<Chunk, Vec<ErrorInfo>> {
    let scanner = Scanner::new(code);
    let frontend = Frontend::new(Box::new(scanner));
    let compiler = frontend.compile()?;
    let chunk = compiler.chunk().clone();
    Ok(chunk)
}
