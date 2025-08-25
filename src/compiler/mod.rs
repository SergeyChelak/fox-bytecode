mod compiler;
mod parser;
mod scanner;
mod token;

use std::rc::Rc;

use scanner::*;
pub use token::*;

use crate::{
    ErrorCollector, compiler::parser::Parser, data::Chunk, errors::ErrorInfo, utils::shared,
};

pub fn compile(code: Rc<Vec<char>>) -> Result<Chunk, Vec<ErrorInfo>> {
    let shared_error_collector = shared(ErrorCollector::new());
    let scanner = Scanner::new(code);
    let parser = Parser::with(Box::new(scanner), shared_error_collector.clone());
    let chunk = parser.compile();
    if shared_error_collector.borrow().has_errors() {
        return Err(shared_error_collector.borrow().errors().clone());
    }
    Ok(chunk)
}
