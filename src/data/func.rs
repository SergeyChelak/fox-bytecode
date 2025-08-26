use std::fmt::Display;

use crate::Chunk;

#[derive(Default, Debug)]
pub struct Func {
    pub(crate) arity: usize,
    chunk: Chunk,
    pub(crate) name: Option<String>,
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
        if let Some(name) = &self.name {
            write!(f, "<fn {name}>")
        } else {
            write!(f, "<script>")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FuncType {
    Script,
    Function,
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Func {
        pub fn any_with_chunk(chunk: Chunk) -> Self {
            Self {
                arity: 0,
                chunk,
                name: None,
            }
        }
    }
}
