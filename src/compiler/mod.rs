mod compiler;
mod frontend;
mod parser;
mod scanner;
mod token;

use std::rc::Rc;

use scanner::*;
pub use token::*;

use crate::{
    ErrorCollector,
    compiler::{compiler::Compiler, parser::Parser},
    data::Chunk,
    errors::ErrorInfo,
    utils::shared,
};

// /////////////////////////////////////////////////
// WARNING: these definitions located here temporary
pub const MAX_SCOPE_SIZE: usize = 256;

pub struct LocalVariableInfo {
    pub index: u8,
    pub depth: Option<usize>,
}

pub struct Local {
    name: String,
    depth: Option<usize>,
}

impl Local {
    pub fn with_name(name: String) -> Self {
        Self { name, depth: None }
    }
}

struct LoopData {
    start: usize,
    breaks: Vec<usize>,
}

impl LoopData {
    fn new(start: usize) -> Self {
        Self {
            start,
            breaks: Default::default(),
        }
    }
}
// WARNING: end
// /////////////////////////////////////////////////

pub fn compile(code: Rc<Vec<char>>) -> Result<Chunk, Vec<ErrorInfo>> {
    let error_collector = shared(ErrorCollector::new());
    let scanner = Scanner::new(code);
    let compiler = Compiler::new(error_collector.clone());
    let parser = Parser::with(Box::new(scanner), error_collector.clone(), compiler);
    let compiler = parser.compile();
    if error_collector.borrow().has_errors() {
        return Err(error_collector.borrow().errors().clone());
    }
    let chunk = compiler.chunk().clone();
    Ok(chunk)
}
