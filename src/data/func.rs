use std::{fmt::Display, rc::Rc};

use crate::{Chunk, Value};

#[derive(Default, Debug)]
pub struct Closure {
    func: Rc<Func>,
}

impl Closure {
    pub fn with(func: Func) -> Self {
        Self::new(Rc::new(func))
    }

    pub fn new(func: Rc<Func>) -> Self {
        Self { func }
    }

    pub fn func(&self) -> &Func {
        &self.func
    }
}

#[derive(Default, Debug)]
pub struct Func {
    pub(crate) arity: usize,
    chunk: Chunk,
    pub(crate) name: Option<String>,
    pub(crate) upvalue_count: usize,
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

pub type NativeFn = fn(&[Value]) -> Value;

#[derive(Debug)]
pub struct NativeFunc {
    func: NativeFn,
}

impl NativeFunc {
    pub fn with(func: NativeFn) -> Self {
        Self { func }
    }

    pub fn call(&self, args: &[Value]) -> Value {
        (self.func)(args)
    }
}

impl Display for NativeFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Func {
        pub fn any_with_chunk(chunk: Chunk) -> Self {
            Self {
                chunk,
                ..Default::default()
            }
        }
    }
}
