mod compiler;
mod frontend;
mod rule;
mod scanner;
mod token;

use std::rc::Rc;

use scanner::*;
pub use token::*;

use crate::{compiler::frontend::Frontend, data::Chunk, errors::ErrorInfo};

pub fn compile(code: Rc<Vec<char>>) -> Result<Chunk, Vec<ErrorInfo>> {
    let scanner = Scanner::new(code);
    // let compiler = Compiler::new();
    // let parser = Parser::with(Box::new(scanner), compiler);
    let frontend = Frontend::new(Box::new(scanner));
    let compiler = frontend.compile()?;
    let chunk = compiler.chunk().clone();
    Ok(chunk)
}
