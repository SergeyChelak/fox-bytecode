mod parser;
mod scanner;
mod token;

use std::rc::Rc;

use scanner::*;
pub use token::*;

use crate::{chunk::Chunk, compiler::parser::Parser, utils::ErrorInfo};

pub fn compile(code: Rc<Vec<char>>) -> Result<Chunk, Vec<ErrorInfo>> {
    let scanner = Scanner::new(code);
    let mut parser = Parser::with(Box::new(scanner));
    parser.compile()?;
    todo!()
}
