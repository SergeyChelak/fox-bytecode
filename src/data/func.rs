use std::fmt::Display;

use crate::Chunk;

#[derive(Default)]
pub struct Func {
    arity: usize,
    chunk: Chunk,
    name: Option<String>,
}

impl Func {
    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunk
    }
}

impl Display for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.as_deref().unwrap_or("**global**");
        write!(f, "<fn {name}>")
    }
}

pub enum FuncType {
    Script,
    Function,
}
