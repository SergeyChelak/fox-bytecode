use std::{
    collections::{HashMap, LinkedList},
    ops::Deref,
    rc::Rc,
};

use crate::{
    MachineError, MachineResult, Shared, StackTraceElement,
    backend::{NativeFunctionsProvider, call_frame::CallFrame, service::BackendService},
    data::*,
    shared,
    utils::bytes_to_word,
};

const FRAMES_MAX: usize = 64;
const STACK_MAX_SIZE: usize = FRAMES_MAX * UINT8_COUNT;

pub struct Machine {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<Rc<String>, Value>,
    service: Shared<dyn BackendService>,
    open_upvalues: LinkedList<Shared<Upvalue>>,
}

impl Machine {
    pub fn with(
        func: Func,
        service: Shared<dyn BackendService>,
        native: impl NativeFunctionsProvider,
    ) -> Self {
        let mut vm = Self::new(service);
        // setup native functions to VM
        native.get_functions().into_iter().for_each(|(name, func)| {
            vm.define_native(name, func);
        });
        // prepare to start
        let func_ref = Rc::new(func);
        let closure = Closure::new(func_ref);
        let closure_ref = Rc::new(closure);
        _ = vm.stack_push(Value::Closure(closure_ref.clone()));
        vm.unchecked_call(closure_ref, 0);
        vm
    }

    fn new(service: Shared<dyn BackendService>) -> Self {
        Self {
            frames: Vec::with_capacity(FRAMES_MAX),
            stack: Vec::with_capacity(STACK_MAX_SIZE),
            globals: HashMap::new(),
            service,
            open_upvalues: Default::default(),
        }
    }

    pub fn run(&mut self) -> MachineResult<()> {
        let result = self.perform();
        if let Err(err) = &result {
            self.service.borrow_mut().set_error(err.clone());
            self.flush_track_trace();
            self.stack.clear();
            self.frames.clear();
        }
        result
    }

    fn perform(&mut self) -> MachineResult<()> {
        let mut is_alive = true;
        while is_alive {
            let fetch_result = self.fetch_instruction();
            let instr = match fetch_result {
                Ok(instr) => instr,
                Err(FetchError::End) => break,
                Err(err) => return Err(self.runtime_error(format!("{err}"))),
            };
            match instr {
                Instruction::Constant(index) => self.op_constant(index)?,
                Instruction::Equal => self.op_binary(Value::equals)?,
                Instruction::Greater => self.op_binary(Value::greater)?,
                Instruction::Less => self.op_binary(Value::less)?,
                Instruction::Nil => self.stack_push(Value::Nil)?,
                Instruction::True => self.stack_push(Value::Bool(true))?,
                Instruction::False => self.stack_push(Value::Bool(false))?,
                Instruction::Add => self.op_binary(Value::add)?,
                Instruction::Subtract => self.op_binary(Value::subtract)?,
                Instruction::Multiply => self.op_binary(Value::multiply)?,
                Instruction::Divide => self.op_binary(Value::divide)?,
                Instruction::Negate => self.op_negate()?,
                Instruction::Not => self.op_not()?,
                Instruction::Print => self.op_print()?,
                Instruction::Return => self.op_return(&mut is_alive)?,
                Instruction::Pop => self.op_pop()?,
                Instruction::DefineGlobal(index) => self.define_global(index)?,
                Instruction::GetGlobal(index) => self.get_global(index)?,
                Instruction::SetGlobal(index) => self.set_global(index)?,
                Instruction::GetLocal(rel_slot) => self.op_get_local(rel_slot)?,
                Instruction::SetLocal(rel_slot) => self.op_set_local(rel_slot)?,
                Instruction::JumpIfFalse(first, second) => self.op_jump_if_false(first, second)?,
                Instruction::Jump(first, second) => self.op_jump(first, second)?,
                Instruction::Loop(first, second) => self.op_loop(first, second)?,
                Instruction::Duplicate => self.op_duplicate_top()?,
                Instruction::Call(arg_count) => self.op_call(arg_count)?,
                Instruction::Closure(index) => self.op_closure(index)?,
                Instruction::GetUpvalue(index) => self.op_get_upvalue(index)?,
                Instruction::SetUpvalue(index) => self.op_set_upvalue(index)?,
                Instruction::CloseUpvalue => self.op_close_upvalue()?,
                Instruction::Class(index) => self.op_class(index)?,
                Instruction::GetProperty(index) => self.get_class_property(index)?,
                Instruction::SetProperty(index) => self.set_class_property(index)?,
                Instruction::Method(index) => self.op_method(index)?,
            }
        }
        Ok(())
    }
}

/// Jumps
impl Machine {
    fn op_return(&mut self, is_alive: &mut bool) -> MachineResult<()> {
        let result = self.stack_pop()?;
        let frame = self
            .frames
            .pop()
            .ok_or(MachineError::with_str("Bug: return on empty call frame"))?;

        if self.frames.is_empty() {
            self.stack_pop()?;
            *is_alive = false;
            return Ok(());
        }

        self.close_upvalues(frame.frame_start())?;
        self.stack.truncate(frame.frame_start());
        self.stack_push(result)
    }

    fn op_loop(&mut self, first: u8, second: u8) -> MachineResult<()> {
        let jump = bytes_to_word(first, second);
        self.frame_mut()?.ip_dec(jump);
        Ok(())
    }

    fn op_jump(&mut self, first: u8, second: u8) -> MachineResult<()> {
        let jump = bytes_to_word(first, second);
        self.frame_mut()?.ip_inc(jump);
        Ok(())
    }

    fn op_jump_if_false(&mut self, first: u8, second: u8) -> MachineResult<()> {
        let jump = bytes_to_word(first, second);
        let condition = self.stack_peek()?.as_bool();
        if !condition {
            self.frame_mut()?.ip_inc(jump);
        }
        Ok(())
    }
}

/// Math and logical ops
impl Machine {
    fn op_not(&mut self) -> MachineResult<()> {
        let value = self.stack_pop()?;
        self.stack_push(Value::Bool(!value.as_bool()))
    }

    fn op_negate(&mut self) -> MachineResult<()> {
        let value = self.stack_pop()?;
        let Some(value) = value.as_number() else {
            return Err(self.runtime_error("Operand must be a number"));
        };
        self.stack_push(Value::number(-value))
    }

    fn op_binary(&mut self, operation: ValueOperation) -> MachineResult<()> {
        let b = self.stack_pop()?;
        let a = self.stack_pop()?;
        match operation(&a, &b) {
            Ok(value) => {
                self.stack.push(value);
                Ok(())
            }
            Err(OperationError::TypeMismatch) => {
                Err(self.runtime_error("Invalid/incompatible operands type"))
            }
            Err(OperationError::DivisionByZero) => Err(self.runtime_error("Division by zeros")),
        }
    }
}

/// Function/calls
impl Machine {
    fn op_call(&mut self, arg_count: u8) -> MachineResult<()> {
        let arg_count = arg_count as usize;
        let value = self.stack_peek_at(arg_count)?;
        self.call_value(value, arg_count)
    }

    fn call_value(&mut self, value: Value, arg_count: usize) -> MachineResult<()> {
        match value {
            Value::Closure(callee) => self.call_closure(callee, arg_count),
            Value::NativeFun(callee) => self.call_native(callee, arg_count),
            Value::Class(callee) => self.instantiate_class(callee, arg_count),
            _ => Err(self.runtime_error("Can only call functions and classes")),
        }
    }

    fn call_closure(&mut self, callee: Rc<Closure>, arg_count: usize) -> MachineResult<()> {
        let arity = callee.func().arity;
        if arg_count != arity {
            let message = format!("Expected {} arguments but got {}", arity, arg_count);
            return Err(self.runtime_error(message));
        }
        if self.frames.len() == FRAMES_MAX {
            return Err(self.runtime_error("Stack overflow"));
        }
        self.unchecked_call(callee, arg_count);
        Ok(())
    }

    fn call_native(&mut self, callee: Rc<NativeFunc>, arg_count: usize) -> MachineResult<()> {
        let len = self.stack.len();
        let args = &self.stack[len - arg_count..];
        let result = callee.call(args);
        self.stack.truncate(len - arg_count);
        self.stack_push(result)
    }

    fn define_native<T: AsRef<str>>(&mut self, name: T, func: NativeFn) {
        let value = Value::native_func(func);
        self.globals
            .insert(Rc::new(name.as_ref().to_string()), value);
    }

    fn unchecked_call(&mut self, closure: Rc<Closure>, arg_count: usize) {
        let frame_start = self.stack.len() - arg_count - 1;
        let frame = CallFrame::new(closure, frame_start);
        self.frames.push(frame);
    }
}

/// Variables
impl Machine {
    fn define_global(&mut self, index: u8) -> MachineResult<()> {
        let name = self.read_const_string(index)?;
        let value = self.stack_pop()?;
        self.globals.insert(name, value);
        Ok(())
    }

    fn get_global(&mut self, index: u8) -> MachineResult<()> {
        let name = self.read_const_string(index)?;
        let Some(value) = self.globals.get(&name).cloned() else {
            let message = format!("Undefined variable {}", name);
            return Err(self.runtime_error(message));
        };
        self.stack_push(value)
    }

    fn set_global(&mut self, index: u8) -> MachineResult<()> {
        let name = self.read_const_string(index)?;
        if !self.globals.contains_key(&name) {
            let message = format!("Undefined variable {}", name);
            return Err(self.runtime_error(message));
        }
        let value = self.stack_peek()?;
        self.globals.insert(name, value);
        Ok(())
    }

    fn op_get_local(&mut self, rel_slot: u8) -> MachineResult<()> {
        let slot = self.relative_to_absolute_slot(rel_slot)?;
        let Some(value) = self.stack.get(slot).cloned() else {
            return Err(self.runtime_error("Bug: failed to get local value"));
        };
        self.stack_push(value)?;
        Ok(())
    }

    fn op_set_local(&mut self, rel_slot: u8) -> MachineResult<()> {
        let value = self.stack_peek()?;
        let slot = self.relative_to_absolute_slot(rel_slot)?;
        self.stack[slot] = value;
        Ok(())
    }

    fn relative_to_absolute_slot(&self, relative_slot: u8) -> MachineResult<usize> {
        let start = self.frame()?.frame_start();
        Ok(start + relative_slot as usize)
    }
}

/// Classes
impl Machine {
    fn instantiate_class(&mut self, callee: Rc<Class>, arg_count: usize) -> MachineResult<()> {
        let len = self.stack.len();
        let instance = Instance::new(callee);
        self.stack[len - arg_count - 1] = Value::Instance(Rc::new(instance));
        Ok(())
    }

    fn op_class(&mut self, index: u8) -> MachineResult<()> {
        let name = self.read_const_string(index)?;
        let class = Class::new(name.clone());
        self.stack_push(Value::Class(Rc::new(class)))
    }

    fn op_method(&mut self, index: u8) -> MachineResult<()> {
        let name = self.read_const_string(index)?;
        self.define_method(name)
    }

    fn define_method(&mut self, name: Rc<String>) -> MachineResult<()> {
        let method = self.stack_peek()?;
        let class = self
            .stack_peek_at(1)?
            .as_class()
            .ok_or(MachineError::with_str("Bug: method on non-class object"))?;
        class.add_method(name, method);
        self.stack_pop()?;
        Ok(())
    }

    fn get_class_property(&mut self, index: u8) -> MachineResult<()> {
        let instance = self
            .stack_peek()?
            .as_instance()
            .ok_or(MachineError::with_str("Only instances have fields"))?;
        let name = self.read_const_string(index)?;
        if let Some(value) = instance.get_field(name.clone()) {
            _ = self.stack_pop()?; // instance
            self.stack_push(value)?;
            return Ok(());
        };

        self.bind_method(instance.class(), name)
    }

    fn bind_method(&mut self, class: Rc<Class>, name: Rc<String>) -> MachineResult<()> {
        let Some(method) = class.get_method(&name) else {
            let msg = format!("Undefined property '{name}'");
            return Err(MachineError::with_str(&msg));
        };
        let closure = method.as_closure().ok_or(MachineError::with_str(
            "Bug: expected closure in bind_method",
        ))?;
        let receiver = self.stack_peek()?;
        let bound = BoundMethod::new(receiver, closure);
        let bound_value = Value::bound(bound);
        _ = self.stack_pop()?;
        self.stack_push(bound_value)
    }

    fn set_class_property(&mut self, index: u8) -> MachineResult<()> {
        let instance = self
            .stack_peek_at(1)?
            .as_instance()
            .ok_or(MachineError::with_str("Only instances have fields"))?;
        let name = self.read_const_string(index)?;
        let value = self.stack_pop()?;
        instance.set_field(name, value.clone());
        _ = self.stack_pop()?;
        self.stack_push(value)
    }
}

/// Closures
impl Machine {
    fn op_closure(&mut self, index: u8) -> MachineResult<()> {
        let val = self.read_const(index)?;
        let func = val.as_function().ok_or(MachineError::with_str(
            "Bug: closure refers to non-function constant",
        ))?;
        let mut closure = Closure::new(func.clone());
        let count = closure.upvalues_count();
        for i in 0..count {
            let data = self
                .frame_mut()?
                .fetch_upvalue_data()
                .ok_or(MachineError::with_str("Bug: missing upvalue"))?;
            let upvalue = if data.is_local {
                self.capture_upvalue(data.index)?
            } else {
                self.frame()?.closure().upvalue(data.index as usize)
            };
            closure.assign_upvalue(i, upvalue);
        }
        let value = Value::Closure(Rc::new(closure));
        self.stack_push(value)
    }

    fn capture_upvalue(&mut self, index: u8) -> MachineResult<Shared<Upvalue>> {
        let index = self.frame()?.frame_start() + index as usize;

        let mut position: Option<usize> = None;
        for (i, val) in self.open_upvalues.iter().enumerate() {
            let stack_idx = extract_stack_index(val)?;
            if stack_idx > index {
                continue;
            }
            if stack_idx == index {
                return Ok(val.clone());
            }
            position = Some(i);
            break;
        }

        let upvalue = shared(Upvalue::Stack(index));
        if let Some(p) = position {
            let mut tail = self.open_upvalues.split_off(p);
            self.open_upvalues.push_back(upvalue.clone());
            self.open_upvalues.append(&mut tail);
        } else {
            self.open_upvalues.push_back(upvalue.clone());
        }
        Ok(upvalue)
    }

    fn op_close_upvalue(&mut self) -> MachineResult<()> {
        self.close_upvalues(self.stack.len() - 1)?;
        self.stack_pop()?;
        Ok(())
    }

    fn close_upvalues(&mut self, last: usize) -> MachineResult<()> {
        loop {
            let Some(front) = self.open_upvalues.front() else {
                break;
            };
            let stack_index = extract_stack_index(front)?;
            if stack_index < last {
                break;
            }
            let value = self.stack[stack_index].clone();
            let upvalue = self
                .open_upvalues
                .pop_front()
                .ok_or(MachineError::with_str("Bug: failed to pop front upvalue"))?;

            *upvalue.borrow_mut() = Upvalue::Heap(shared(value));
        }
        Ok(())
    }

    fn op_get_upvalue(&mut self, index: u8) -> MachineResult<()> {
        let shared_upvalue = self.frame()?.closure().upvalue(index as usize);
        let upvalue = shared_upvalue
            .try_borrow()
            .map_err(|err| MachineError::with_str(&err.to_string()))?;
        let value = match upvalue.deref() {
            Upvalue::Stack(index) => self.stack_get(*index)?,
            Upvalue::Heap(ref_cell) => ref_cell.borrow().clone(),
            Upvalue::Nil => {
                return Err(MachineError::with_str("Bug: get_upvalue got nil"));
            }
        };
        self.stack_push(value)
    }

    fn op_set_upvalue(&mut self, index: u8) -> MachineResult<()> {
        let shared_upvalue = self.frame()?.closure().upvalue(index as usize);
        let upvalue = shared_upvalue
            .try_borrow()
            .map_err(|err| MachineError::with_str(&err.to_string()))?;

        let value = self.stack_peek()?;
        match upvalue.deref() {
            Upvalue::Stack(index) => {
                self.stack[*index] = value;
            }
            Upvalue::Heap(ref_cell) => {
                *ref_cell.borrow_mut() = value;
            }
            Upvalue::Nil => {
                return Err(MachineError::with_str("Bug: set_upvalue got nil"));
            }
        };
        Ok(())
    }
}

/// Access & fetch
impl Machine {
    fn op_constant(&mut self, index: u8) -> MachineResult<()> {
        let value = self.read_const(index)?;
        self.stack_push(value)
    }

    fn fetch_instruction(&mut self) -> FetchResult<Instruction> {
        let frame = self
            .frame_mut()
            .map_err(|err| FetchError::Other(err.text))?;
        frame.fetch_instruction()
    }

    fn frame(&self) -> MachineResult<&CallFrame> {
        let Some(f) = self.frames.last() else {
            return Err(MachineError::with_str("Bug: empty call frame"));
        };
        Ok(f)
    }

    fn frame_mut(&mut self) -> MachineResult<&mut CallFrame> {
        let Some(f) = self.frames.last_mut() else {
            return Err(MachineError::with_str("Bug: empty call frame"));
        };
        Ok(f)
    }

    fn read_const(&self, index: u8) -> MachineResult<Value> {
        let Some(value) = self.frame()?.chunk().read_const(index) else {
            return Err(self.runtime_error("Invalid constant index"));
        };
        Ok(value)
    }

    fn read_const_string(&self, index: u8) -> MachineResult<Rc<String>> {
        let name = self.read_const(index)?;
        let Some(name) = name.as_text() else {
            return Err(self.runtime_error("Bug: failed to fetch constant"));
        };
        Ok(name)
    }
}

/// Stack
impl Machine {
    fn op_duplicate_top(&mut self) -> MachineResult<()> {
        let value = self.stack_peek()?;
        self.stack_push(value)
    }

    fn op_pop(&mut self) -> MachineResult<()> {
        self.stack_pop()?;
        Ok(())
    }

    fn stack_push(&mut self, value: Value) -> MachineResult<()> {
        if self.stack.len() >= STACK_MAX_SIZE {
            return Err(self.runtime_error("Stack overflow"));
        }
        self.stack.push(value);
        Ok(())
    }

    fn stack_peek(&self) -> MachineResult<Value> {
        self.stack_peek_at(0)
    }

    fn stack_get(&self, index: usize) -> MachineResult<Value> {
        self.stack
            .get(index)
            .cloned()
            .ok_or(MachineError::with_str(&format!(
                "Bug: invalid stack index ({index})"
            )))
    }

    fn stack_peek_at(&self, rev_index: usize) -> MachineResult<Value> {
        let len = self.stack.len();
        let err = || {
            let msg = format!("Bug: trying access stack with invalid index {rev_index}");
            Err(MachineError::with_str(msg.as_str()))
        };
        if rev_index >= len {
            return err();
        }
        let Some(value) = self.stack.get(len - rev_index - 1).cloned() else {
            return err();
        };
        Ok(value)
    }

    fn stack_pop(&mut self) -> MachineResult<Value> {
        let Some(value) = self.stack.pop() else {
            return Err(self.runtime_error("Pop on empty stack"));
        };
        Ok(value)
    }
}

fn extract_stack_index(val: &Shared<Upvalue>) -> MachineResult<usize> {
    let Upvalue::Stack(stack_idx) = *val.borrow().deref() else {
        return Err(MachineError::with_str(
            "Bug: open upvalues should be stack allocated",
        ));
    };
    Ok(stack_idx)
}

/// Errors & utils
impl Machine {
    fn op_print(&mut self) -> MachineResult<()> {
        let value = self.stack_pop()?;
        self.service.borrow_mut().print_value(value);
        Ok(())
    }

    fn runtime_error<T: AsRef<str>>(&self, message: T) -> MachineError {
        let mut line_number: Option<usize> = None;
        if let Ok(frame) = self.frame() {
            let idx = frame.ip() - 1;
            line_number = frame.chunk().line_number(idx);
        }
        MachineError {
            text: message.as_ref().to_string(),
            line_number,
        }
    }

    fn flush_track_trace(&mut self) {
        let stack_trace = self
            .frames
            .iter()
            .rev()
            .map(|frame| StackTraceElement {
                line: frame.line_number(),
                func_name: frame.func_name().map(|s| s.to_string()),
            })
            .collect::<Vec<_>>();
        self.service.borrow_mut().set_stack_trace(stack_trace);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        backend::{service::probe::ProbeBackendService, *},
        utils::{Shared, shared},
    };

    #[test]
    fn operation_negate() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_NEGATE, 1);
        machine_test(chunk, &[Value::number(10.0)], &[Value::number(-10.0)], &[])
    }

    #[test]
    fn operation_not() -> MachineResult<()> {
        let make_chunk = || {
            let mut chunk = Chunk::new();
            chunk.write_u8(OPCODE_NOT, 0);
            chunk
        };
        machine_test(
            make_chunk(),
            &[Value::Bool(true)],
            &[Value::Bool(false)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::Bool(false)],
            &[Value::Bool(true)],
            &[],
        )
    }

    #[test]
    fn operation_equal() -> MachineResult<()> {
        let make_chunk = || {
            let mut chunk = Chunk::new();
            chunk.write_u8(OPCODE_EQUAL, 1);
            chunk
        };
        machine_test(
            make_chunk(),
            &[Value::number(2.0), Value::number(2.0)],
            &[Value::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::number(3.0), Value::number(2.0)],
            &[Value::Bool(false)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::Bool(false), Value::Bool(false)],
            &[Value::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::Nil, Value::Nil],
            &[Value::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::Bool(false), Value::Nil],
            &[Value::Bool(false)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::text_from_str("abc"), Value::text_from_str("abc")],
            &[Value::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::text_from_str("abc"), Value::text_from_str("abcd")],
            &[Value::Bool(false)],
            &[],
        )
    }

    #[test]
    fn operation_greater() -> MachineResult<()> {
        let make_chunk = || {
            let mut chunk = Chunk::new();
            chunk.write_u8(OPCODE_GREATER, 1);
            chunk
        };
        machine_test(
            make_chunk(),
            &[Value::number(3.0), Value::number(2.0)],
            &[Value::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::number(1.0), Value::number(2.0)],
            &[Value::Bool(false)],
            &[],
        )
    }

    #[test]
    fn operation_less() -> MachineResult<()> {
        let make_chunk = || {
            let mut chunk = Chunk::new();
            chunk.write_u8(OPCODE_LESS, 1);
            chunk
        };
        machine_test(
            make_chunk(),
            &[Value::number(3.0), Value::number(2.0)],
            &[Value::Bool(false)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[Value::number(1.0), Value::number(2.0)],
            &[Value::Bool(true)],
            &[],
        )?;

        let res = machine_test(
            make_chunk(),
            &[Value::number(1.0), Value::Bool(true)],
            &[],
            &[],
        );
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn operation_nil() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_NIL, 1);
        machine_test(chunk, &[], &[Value::Nil], &[])
    }

    #[test]
    fn operation_true() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_TRUE, 1);
        machine_test(chunk, &[], &[Value::Bool(true)], &[])
    }

    #[test]
    fn operation_false() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_FALSE, 1);
        machine_test(chunk, &[], &[Value::Bool(false)], &[])
    }

    #[test]
    fn operation_pop() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_POP, 1);
        machine_test(chunk, &[Value::Bool(true)], &[], &[])
    }

    #[test]
    fn operation_add() -> MachineResult<()> {
        let make_chunk = || {
            let mut chunk = Chunk::new();
            chunk.write_u8(OPCODE_ADD, 1);
            chunk
        };
        machine_test(
            make_chunk(),
            &[Value::number(2.0), Value::number(3.0)],
            &[Value::number(5.0)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[
                Value::text_from_str("first"),
                Value::text_from_str("_second"),
            ],
            &[Value::text_from_str("first_second")],
            &[],
        )
    }

    #[test]
    fn operation_sub() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_SUBTRACT, 1);
        machine_test(
            chunk,
            &[Value::number(2.0), Value::number(3.0)],
            &[Value::number(-1.0)],
            &[],
        )
    }

    #[test]
    fn operation_mul() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_MULTIPLY, 1);
        machine_test(
            chunk,
            &[Value::number(2.0), Value::number(3.0)],
            &[Value::number(6.0)],
            &[],
        )
    }

    #[test]
    fn operation_div() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_DIVIDE, 1);
        machine_test(
            chunk,
            &[Value::number(6.0), Value::number(3.0)],
            &[Value::number(2.0)],
            &[],
        )
    }

    #[test]
    fn operation_duplicate() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_DUPLICATE, 1);
        machine_test(
            chunk,
            &[Value::number(6.0)],
            &[Value::number(6.0), Value::number(6.0)],
            &[],
        )
    }

    #[test]
    fn operation_print() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_PRINT, 1);
        machine_test(
            chunk,
            &[Value::text_from_str("abc")],
            &[],
            &["abc".to_string()],
        )
    }

    #[test]
    fn peek_test() -> MachineResult<()> {
        let chunk = Chunk::new();
        let probe_ref = make_probe_ref();
        let mut vm = make_machine(chunk, probe_ref);

        let a = Value::Number(1.0);
        let b = Value::Number(2.0);
        let c = Value::Number(3.0);
        vm.stack_push(a.clone())?;
        vm.stack_push(b.clone())?;
        vm.stack_push(c.clone())?;
        // random picks
        assert_eq!(vm.stack_peek_at(2)?, a);
        assert_eq!(vm.stack_peek()?, c);
        assert_eq!(vm.stack_peek_at(1)?, b);
        // main function pushed at zero position, peek at index 4 for error
        assert!(vm.stack_peek_at(4).is_err());
        Ok(())
    }

    #[test]
    fn operation_constant() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_CONSTANT, 1);
        chunk.add_constant(Value::number(2.0));
        let idx = chunk.add_constant(Value::number(10.0));
        chunk.write_u8(idx as u8, 1);
        let mut machine = make_machine(chunk, make_probe_ref());
        machine.run()?;
        assert_eq!(machine.stack_pop().unwrap().as_number(), Some(10.0));
        Ok(())
    }

    fn machine_test(
        chunk: Chunk,
        stack_in: &[Value],
        stack_out: &[Value],
        buffer_out: &[String],
    ) -> MachineResult<()> {
        let probe_ref = make_probe_ref();
        let mut machine = make_machine(chunk, probe_ref.clone());
        for v_in in stack_in {
            machine.stack_push(v_in.clone())?;
        }
        machine.run()?;
        for v_out in stack_out.iter().rev() {
            let val = machine.stack_pop()?;
            assert_eq!(val, *v_out);
        }

        let probe = probe_ref.borrow();
        probe.assert_output_match(buffer_out);

        // TODO: maybe drop this check
        assert_eq!(machine.stack.len(), 1);
        Ok(())
    }

    fn make_probe_ref() -> Shared<ProbeBackendService> {
        let probe_service = ProbeBackendService::default();
        shared(probe_service)
    }

    fn make_machine(chunk: Chunk, backend: Shared<dyn BackendService>) -> Machine {
        let func = Func::any_with_chunk(chunk);
        Machine::with(func, backend, EmptyNative)
    }
}
