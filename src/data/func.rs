use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{Chunk, Shared, Value, shared};

#[derive(Default)]
pub struct Closure {
    func: Rc<Func>,
    upvalues: Vec<Shared<Upvalue>>,
}

impl Debug for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Closure")
            .field("func", &self.func)
            .field("upvalues_count", &self.upvalues.len())
            .finish()
    }
}

impl Closure {
    pub fn new(func: Rc<Func>) -> Self {
        let count = func.upvalue_count;
        let upvalues = vec![shared(Upvalue::Nil); count];
        Self { func, upvalues }
    }

    pub fn func(&self) -> &Func {
        &self.func
    }

    pub fn upvalues_count(&self) -> usize {
        self.upvalues.len()
    }

    pub fn upvalue(&self, index: usize) -> Shared<Upvalue> {
        self.upvalues[index].clone()
    }

    pub fn assign_upvalue(&mut self, index: usize, upvalue: Shared<Upvalue>) {
        self.upvalues[index] = upvalue;
    }
}

pub enum Upvalue {
    Stack(usize),
    Heap(Shared<Value>),
    Nil,
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
