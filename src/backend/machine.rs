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
        {
            let func_val = Value::Fun(func_ref.clone());
            _ = vm.stack_push(func_val);
        }

        let closure = Closure::new(func_ref);
        let closure_ref = Rc::new(closure);
        _ = vm.stack_pop();
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
            self.stack_reset();
        }
        result
    }

    fn perform(&mut self) -> MachineResult<()> {
        'run_loop: loop {
            let fetch_result = self.fetch_instruction();
            let instr = match fetch_result {
                Ok(instr) => instr,
                Err(FetchError::End) => break,
                Err(err) => return Err(self.runtime_error(format!("{err}"))),
            };
            match instr {
                Instruction::Constant(index) => {
                    let value = self.read_const(index)?;
                    self.stack_push(value)?;
                }
                Instruction::Equal => self.do_binary(Value::equals)?,
                Instruction::Greater => self.do_binary(Value::greater)?,
                Instruction::Less => self.do_binary(Value::less)?,
                Instruction::Nil => self.stack_push(Value::Nil)?,
                Instruction::True => self.stack_push(Value::Bool(true))?,
                Instruction::False => self.stack_push(Value::Bool(false))?,
                Instruction::Add => self.do_binary(Value::add)?,
                Instruction::Subtract => self.do_binary(Value::subtract)?,
                Instruction::Multiply => self.do_binary(Value::multiply)?,
                Instruction::Divide => self.do_binary(Value::divide)?,
                Instruction::Negate => {
                    let value = self.stack_pop()?;
                    let Some(value) = value.as_number() else {
                        return Err(self.runtime_error("Operand must be a number"));
                    };
                    self.stack_push(Value::number(-value))?;
                }
                Instruction::Not => {
                    let value = self.stack_pop()?;
                    self.stack_push(Value::Bool(!value.as_bool()))?;
                }
                Instruction::Print => {
                    let value = self.stack_pop()?;
                    self.service.borrow_mut().print_value(value);
                }
                Instruction::Return => {
                    let result = self.stack_pop()?;
                    let Some(frame) = self.frames.pop() else {
                        return Err(MachineError::with_str("Bug: return on empty call frame"));
                    };

                    if self.frames.is_empty() {
                        self.stack_pop()?;
                        break 'run_loop;
                    }

                    self.stack.truncate(frame.frame_start());
                    self.stack_push(result)?;
                }
                Instruction::Pop => {
                    self.stack_pop()?;
                }
                Instruction::DefineGlobal(index) => self.define_global(index)?,
                Instruction::GetGlobal(index) => self.get_global(index)?,
                Instruction::SetGlobal(index) => self.set_global(index)?,
                Instruction::GetLocal(rel_slot) => {
                    let slot = self.relative_to_absolute_slot(rel_slot)?;
                    let Some(value) = self.stack.get(slot).cloned() else {
                        let msg = format!("Bug: failed to get local value with '{:?}'", instr);
                        return Err(self.runtime_error(msg));
                    };
                    self.stack_push(value)?
                }
                Instruction::SetLocal(rel_slot) => {
                    let value = self.stack_peek()?;
                    let slot = self.relative_to_absolute_slot(rel_slot)?;
                    self.stack[slot] = value;
                }
                Instruction::JumpIfFalse(first, second) => {
                    let jump = bytes_to_word(first, second);
                    let condition = self.stack_peek()?.as_bool();
                    if !condition {
                        self.frame_mut()?.ip_inc(jump);
                    }
                }
                Instruction::Jump(first, second) => {
                    let jump = bytes_to_word(first, second);
                    self.frame_mut()?.ip_inc(jump);
                }
                Instruction::Loop(first, second) => {
                    let jump = bytes_to_word(first, second);
                    self.frame_mut()?.ip_dec(jump);
                }
                Instruction::Duplicate => {
                    let value = self.stack_peek()?;
                    self.stack_push(value)?;
                }
                Instruction::Call(arg_count) => {
                    let arg_count = arg_count as usize;
                    let value = self.stack_peek_at(arg_count)?;
                    self.call_value(value, arg_count)?;
                }
                Instruction::Closure(index) => self.compose_closure(index)?,
                Instruction::GetUpvalue(index) => self.get_upvalue(index)?,
                Instruction::SetUpvalue(index) => self.set_upvalue(index)?,
                Instruction::CloseUpvalue => todo!(),
            }
        }
        Ok(())
    }

    fn compose_closure(&mut self, index: u8) -> MachineResult<()> {
        let val = self.read_const(index)?;
        let Some(func) = &val.as_function() else {
            return Err(MachineError::with_str(
                "Bug: closure refers to non-function constant",
            ));
        };
        let mut closure = Closure::new(func.clone());
        let count = closure.upvalues_count();
        for i in 0..count {
            let Some(data) = self.frame_mut()?.fetch_upvalue_data() else {
                return Err(MachineError::with_str("Bug: missing upvalue"));
            };
            let upvalue = if data.is_local {
                self.capture_upvalue(data.index)
            } else {
                self.frame()?.closure().upvalue(data.index as usize)
            };
            closure.assign_upvalue(i, upvalue);
        }
        let value = Value::Closure(Rc::new(closure));
        self.stack_push(value)
    }

    fn capture_upvalue(&mut self, index: u8) -> Shared<Upvalue> {
        let index = index as usize;

        let mut position: Option<usize> = None;
        for (i, val) in self.open_upvalues.iter().enumerate() {
            let Upvalue::Stack(stack_idx) = *val.borrow().deref() else {
                unreachable!("Bug: open upvalues should be stack allocated");
            };
            if stack_idx > index {
                continue;
            }
            if stack_idx == index {
                return val.clone();
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
        upvalue
    }

    fn get_upvalue(&mut self, index: u8) -> MachineResult<()> {
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

    fn set_upvalue(&mut self, index: u8) -> MachineResult<()> {
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

    fn read_const_string(&self, index: u8) -> MachineResult<Rc<String>> {
        let name = self.read_const(index)?;
        let Some(name) = name.as_text() else {
            return Err(self.runtime_error("Bug: failed to fetch constant"));
        };
        Ok(name)
    }

    fn do_binary(&mut self, operation: ValueOperation) -> MachineResult<()> {
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

    fn read_const(&self, index: u8) -> MachineResult<Value> {
        let Some(value) = self.frame()?.chunk().read_const(index) else {
            return Err(self.runtime_error("Invalid constant index"));
        };
        Ok(value)
    }

    fn stack_reset(&mut self) {
        self.stack.clear();
        self.frames.clear();
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
            .ok_or(MachineError::with_str("Bug: invalid stack index"))
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

    fn call_value(&mut self, value: Value, arg_count: usize) -> MachineResult<()> {
        match value {
            Value::Closure(callee) => self.call(callee, arg_count),
            Value::NativeFun(callee) => self.call_native(callee, arg_count),
            _ => Err(self.runtime_error("Can only call functions and classes")),
        }
    }

    fn call(&mut self, closure: Rc<Closure>, arg_count: usize) -> MachineResult<()> {
        let arity = closure.func().arity;
        if arg_count != arity {
            let message = format!("Expected {} arguments but got {}", arity, arg_count);
            return Err(self.runtime_error(message));
        }
        if self.frames.len() == FRAMES_MAX {
            return Err(self.runtime_error("Stack overflow"));
        }
        self.unchecked_call(closure, arg_count);
        Ok(())
    }

    fn call_native(&mut self, native_ref: Rc<NativeFunc>, arg_count: usize) -> MachineResult<()> {
        let len = self.stack.len();
        let args = &self.stack[len - arg_count..];
        let result = native_ref.call(args);
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

    fn relative_to_absolute_slot(&self, relative_slot: u8) -> MachineResult<usize> {
        let start = self.frame()?.frame_start();
        Ok(start + relative_slot as usize)
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
