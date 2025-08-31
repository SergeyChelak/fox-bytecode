use std::rc::Rc;

use crate::{Chunk, Closure, FetchResult, Func, Instruction, UpvalueData};

pub struct CallFrame {
    closure: Rc<Closure>,
    ip: usize,
    frame_start: usize,
}

impl CallFrame {
    pub fn new(closure: Rc<Closure>, frame_start: usize) -> Self {
        Self {
            closure,
            ip: 0,
            frame_start,
        }
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn frame_start(&self) -> usize {
        self.frame_start
    }

    pub fn ip_inc(&mut self, val: usize) {
        self.ip += val;
    }

    pub fn ip_dec(&mut self, val: usize) {
        self.ip -= val;
    }

    fn func(&self) -> &Func {
        self.closure.func()
    }

    pub fn closure(&self) -> &Closure {
        &self.closure
    }

    pub fn chunk(&self) -> &Chunk {
        self.func().chunk()
    }

    pub fn line_number(&self) -> Option<usize> {
        self.chunk().line_number(self.ip)
    }

    pub fn fetch_instruction(&mut self) -> FetchResult<Instruction> {
        self.closure.func().chunk().fetch(&mut self.ip)
    }

    pub fn fetch_upvalue_data(&mut self) -> Option<UpvalueData> {
        self.closure.func().chunk().upvalue_data(&mut self.ip)
    }

    pub fn func_name(&self) -> Option<&str> {
        self.closure.func().name.as_deref()
    }
}
