use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{MachineError, MachineResult, backend::MachineIO, data::*, utils::bytes_to_jump};

const FRAMES_MAX: usize = 64;
const STACK_MAX_SIZE: usize = FRAMES_MAX * UINT8_COUNT;

struct CallFrame {
    func_ref: Rc<Func>,
    ip: usize,
    frame_start: usize,
}

impl CallFrame {
    fn ip_inc(&mut self, val: usize) {
        self.ip += val;
    }

    fn ip_dec(&mut self, val: usize) {
        self.ip -= val;
    }
}

pub struct Machine {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<Rc<String>, Value>,
    io: Rc<RefCell<dyn MachineIO>>,
}

impl Machine {
    pub fn with(func: Func, io: Rc<RefCell<dyn MachineIO>>) -> Self {
        let mut vm = Self::new(io);
        let func_ref = Rc::new(func);
        _ = vm.stack_push(Value::Fun(func_ref.clone()));
        vm.call(func_ref, 0);
        vm
    }

    fn new(io: Rc<RefCell<dyn MachineIO>>) -> Self {
        Self {
            frames: Vec::with_capacity(FRAMES_MAX),
            stack: Vec::with_capacity(STACK_MAX_SIZE),
            globals: HashMap::new(),
            io,
        }
    }

    pub fn run(&mut self) -> MachineResult<()> {
        let result = self.perform();
        if let Err(err) = &result {
            self.stack_reset();
            self.io.borrow_mut().set_vm_error(err.clone());
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
                    self.io.borrow_mut().push_output(value);
                }
                Instruction::Return => {
                    break 'run_loop;
                }
                Instruction::Pop => {
                    self.stack_pop()?;
                }
                Instruction::DefineGlobal(index) => self.define_global(index)?,
                Instruction::GetGlobal(index) => self.get_global(index)?,
                Instruction::SetGlobal(index) => self.set_global(index)?,
                Instruction::GetLocal(rel_slot) => {
                    let slot = self.relative_to_absolute_slot(rel_slot)?;
                    let Some(value) = self.stack.get(slot as usize).cloned() else {
                        let msg = format!("Bug: failed to get local value with '{:?}'", instr);
                        return Err(self.runtime_error(msg));
                    };
                    self.stack_push(value)?
                }
                Instruction::SetLocal(rel_slot) => {
                    let Some(value) = self.stack_peek() else {
                        return Err(self.runtime_error("Bug: empty stack on 'SetLocal'"));
                    };
                    let slot = self.relative_to_absolute_slot(rel_slot)?;
                    self.stack[slot as usize] = value;
                }
                Instruction::JumpIfFalse(first, second) => {
                    let jump = bytes_to_jump(first, second);
                    let Some(condition) = self.stack_peek().map(|val| val.as_bool()) else {
                        return Err(self.runtime_error("Bug: empty stack on 'JumpIfFalse'"));
                    };
                    if !condition {
                        self.frame_mut()?.ip_inc(jump);
                        // self.ip += jump;
                    }
                }
                Instruction::Jump(first, second) => {
                    let jump = bytes_to_jump(first, second);
                    // self.ip += jump;
                    self.frame_mut()?.ip_inc(jump);
                }
                Instruction::Loop(first, second) => {
                    let jump = bytes_to_jump(first, second);
                    // self.ip -= jump;
                    self.frame_mut()?.ip_dec(jump);
                }
                Instruction::Duplicate => {
                    let Some(value) = self.stack_peek().clone() else {
                        return Err(self.runtime_error("Bug: empty stack on 'Duplicate' call"));
                    };
                    self.stack_push(value)?;
                }
                Instruction::Call(_args_count) => todo!(),
            }
        }
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
            return Err(self.runtime_error(&message));
        };
        self.stack_push(value)
    }

    fn set_global(&mut self, index: u8) -> MachineResult<()> {
        let name = self.read_const_string(index)?;
        if !self.globals.contains_key(&name) {
            let message = format!("Undefined variable {}", name);
            return Err(self.runtime_error(&message));
        }
        let Some(value) = self.stack_peek() else {
            let message = format!("Bug: not value for '{}' variable", name);
            return Err(self.runtime_error(&message));
        };
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
        let Some(value) = self.chunk()?.read_const(index) else {
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

    fn stack_peek(&mut self) -> Option<Value> {
        self.stack.last().cloned()
    }

    fn stack_pop(&mut self) -> MachineResult<Value> {
        let Some(value) = self.stack.pop() else {
            return Err(self.runtime_error("Pop on empty stack"));
        };
        Ok(value)
    }

    fn call(&mut self, func_ref: Rc<Func>, arg_count: usize) {
        let frame = CallFrame {
            func_ref,
            ip: 0,
            frame_start: self.stack.len() - arg_count - 1,
        };
        self.frames.push(frame);
    }

    fn fetch_instruction(&mut self) -> FetchResult<Instruction> {
        let frame = self
            .frame_mut()
            .map_err(|err| FetchError::Other(err.text))?;
        frame.func_ref.chunk().fetch(&mut frame.ip)
    }

    fn frame(&self) -> MachineResult<&CallFrame> {
        let Some(f) = self.frames.last() else {
            return Err(MachineError::with_str("Bug: empty call frame"));
        };
        Ok(f)
    }

    fn chunk(&self) -> MachineResult<&Chunk> {
        Ok(&self.frame()?.func_ref.chunk())
    }

    fn frame_mut(&mut self) -> MachineResult<&mut CallFrame> {
        let Some(f) = self.frames.last_mut() else {
            return Err(MachineError::with_str("Bug: empty call frame"));
        };
        return Ok(f);
    }

    fn relative_to_absolute_slot(&self, relative_slot: u8) -> MachineResult<usize> {
        let start = self.frame()?.frame_start;
        Ok(start + relative_slot as usize)
    }

    fn runtime_error<T: AsRef<str>>(&self, message: T) -> MachineError {
        let mut line_number: Option<usize> = None;
        if let Ok(frame) = self.frame()
            && let Ok(chunk) = self.chunk()
        {
            let idx = frame.ip - 1;
            line_number = chunk.line_number(idx);
        }
        MachineError {
            text: message.as_ref().to_string(),
            line_number,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backend::*;

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
    fn operation_constant() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_CONSTANT, 1);
        chunk.add_constant(Value::number(2.0));
        let idx = chunk.add_constant(Value::number(10.0));
        chunk.write_u8(idx as u8, 1);
        let func = Func::any_with_chunk(chunk);
        let mut machine = Machine::with(func, Rc::new(RefCell::new(DummyIO)));
        machine.run()?;
        assert_eq!(machine.stack_pop().unwrap().as_number(), Some(10.0));
        Ok(())
    }

    struct DummyIO;

    impl MachineIO for DummyIO {
        fn push_output(&mut self, _value: Value) {
            // no op
        }

        fn set_vm_error(&mut self, _error: MachineError) {
            // no op
        }

        fn set_scanner_errors(&mut self, _errors: &[ErrorInfo]) {
            // no op
        }
    }

    fn machine_test(
        chunk: Chunk,
        stack_in: &[Value],
        stack_out: &[Value],
        buffer_out: &[String],
    ) -> MachineResult<()> {
        let probe_ref = Rc::new(RefCell::new(Probe::new()));
        let func = Func::any_with_chunk(chunk);
        let mut machine = Machine::with(func, probe_ref.clone());
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
}
