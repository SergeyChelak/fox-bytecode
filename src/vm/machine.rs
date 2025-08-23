use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    MachineError, MachineResult,
    chunk::Chunk,
    data::{DataOperation, DataType, OperationError},
    vm::{FetchError, Instruction, MachineIO},
};

const STACK_MAX_SIZE: usize = 256;

pub struct Machine {
    chunk: Chunk,
    ip: usize,
    stack: Vec<DataType>,
    globals: HashMap<Rc<String>, DataType>,
    io: Rc<RefCell<dyn MachineIO>>,
}

impl Machine {
    pub fn with(chunk: Chunk, io: Rc<RefCell<dyn MachineIO>>) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::<DataType>::with_capacity(STACK_MAX_SIZE),
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
            // let ip = self.ip;
            let fetch_result = self.chunk.fetch(&mut self.ip);
            let instr = match fetch_result {
                Ok(instr) => instr,
                Err(FetchError::End) => break,
                Err(err) => return Err(self.runtime_error(format!("{err}"))),
            };
            // println!("{}", self.chunk.disassemble_instruction(&instr, ip));
            match instr {
                Instruction::Constant(index) => {
                    let value = self.read_const(index)?;
                    self.stack_push(value)?;
                }
                Instruction::Equal => self.do_binary(DataType::equals)?,
                Instruction::Greater => self.do_binary(DataType::greater)?,
                Instruction::Less => self.do_binary(DataType::less)?,
                Instruction::Nil => self.stack_push(DataType::Nil)?,
                Instruction::True => self.stack_push(DataType::Bool(true))?,
                Instruction::False => self.stack_push(DataType::Bool(false))?,
                Instruction::Add => self.do_binary(DataType::add)?,
                Instruction::Subtract => self.do_binary(DataType::subtract)?,
                Instruction::Multiply => self.do_binary(DataType::multiply)?,
                Instruction::Divide => self.do_binary(DataType::divide)?,
                Instruction::Negate => {
                    let value = self.stack_pop()?;
                    let Some(value) = value.as_number() else {
                        return Err(self.runtime_error("Operand must be a number"));
                    };
                    self.stack_push(DataType::number(-value))?;
                }
                Instruction::Not => {
                    let value = self.stack_pop()?;
                    self.stack_push(DataType::Bool(!value.as_bool()))?;
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
                Instruction::GetLocal(slot) => {
                    let value = self.stack[slot as usize].clone();
                    self.stack_push(value)?
                }
                Instruction::SetLocal(slot) => {
                    let Some(value) = self.stack_peek() else {
                        return Err(self.runtime_error("Bug: empty stack on 'SetLocal'"));
                    };
                    self.stack[slot as usize] = value;
                }
                Instruction::JumpIfFalse(low, high) => {
                    let jump = ((low as usize) << 8) | (high as usize);
                    let Some(condition) = self.stack_peek().map(|val| val.as_bool()) else {
                        return Err(self.runtime_error("Bug: empty stack on 'JIF'"));
                    };
                    if !condition {
                        self.ip += jump;
                    }
                }
                Instruction::Jump(low, high) => {
                    let jump = ((low as usize) << 8) | (high as usize);
                    self.ip += jump;
                }
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

    fn do_binary(&mut self, operation: DataOperation) -> MachineResult<()> {
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

    fn read_const(&self, index: u8) -> MachineResult<DataType> {
        let Some(value) = self.chunk.read_const(index) else {
            return Err(self.runtime_error("Invalid constant index"));
        };
        Ok(value)
    }

    fn stack_reset(&mut self) {
        self.stack.clear();
    }

    fn stack_push(&mut self, value: DataType) -> MachineResult<()> {
        if self.stack.len() >= STACK_MAX_SIZE {
            return Err(self.runtime_error("Stack overflow"));
        }
        self.stack.push(value);
        Ok(())
    }

    fn stack_peek(&mut self) -> Option<DataType> {
        self.stack.last().cloned()
    }

    fn stack_pop(&mut self) -> MachineResult<DataType> {
        let Some(value) = self.stack.pop() else {
            return Err(self.runtime_error("Pop on empty stack"));
        };
        Ok(value)
    }

    fn runtime_error<T: AsRef<str>>(&self, message: T) -> MachineError {
        let idx = self.ip - 1;
        let line_number = self.chunk.line_number(idx);
        MachineError {
            text: message.as_ref().to_string(),
            line_number,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::vm::*;

    #[test]
    fn operation_negate() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_NEGATE, 1);
        machine_test(
            chunk,
            &[DataType::number(10.0)],
            &[DataType::number(-10.0)],
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
            &[DataType::number(2.0), DataType::number(2.0)],
            &[DataType::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::number(3.0), DataType::number(2.0)],
            &[DataType::Bool(false)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::Bool(false), DataType::Bool(false)],
            &[DataType::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::Nil, DataType::Nil],
            &[DataType::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::Bool(false), DataType::Nil],
            &[DataType::Bool(false)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[
                DataType::text_from_str("abc"),
                DataType::text_from_str("abc"),
            ],
            &[DataType::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[
                DataType::text_from_str("abc"),
                DataType::text_from_str("abcd"),
            ],
            &[DataType::Bool(false)],
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
            &[DataType::number(3.0), DataType::number(2.0)],
            &[DataType::Bool(true)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::number(1.0), DataType::number(2.0)],
            &[DataType::Bool(false)],
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
            &[DataType::number(3.0), DataType::number(2.0)],
            &[DataType::Bool(false)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::number(1.0), DataType::number(2.0)],
            &[DataType::Bool(true)],
            &[],
        )?;

        let res = machine_test(
            make_chunk(),
            &[DataType::number(1.0), DataType::Bool(true)],
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
        machine_test(chunk, &[], &[DataType::Nil], &[])
    }

    #[test]
    fn operation_true() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_TRUE, 1);
        machine_test(chunk, &[], &[DataType::Bool(true)], &[])
    }

    #[test]
    fn operation_false() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_FALSE, 1);
        machine_test(chunk, &[], &[DataType::Bool(false)], &[])
    }

    #[test]
    fn operation_pop() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_POP, 1);
        machine_test(chunk, &[DataType::Bool(true)], &[], &[])
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
            &[DataType::number(2.0), DataType::number(3.0)],
            &[DataType::number(5.0)],
            &[],
        )?;
        machine_test(
            make_chunk(),
            &[
                DataType::text_from_str("first"),
                DataType::text_from_str("_second"),
            ],
            &[DataType::text_from_str("first_second")],
            &[],
        )
    }

    #[test]
    fn operation_sub() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_SUBTRACT, 1);
        machine_test(
            chunk,
            &[DataType::number(2.0), DataType::number(3.0)],
            &[DataType::number(-1.0)],
            &[],
        )
    }

    #[test]
    fn operation_mul() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_MULTIPLY, 1);
        machine_test(
            chunk,
            &[DataType::number(2.0), DataType::number(3.0)],
            &[DataType::number(6.0)],
            &[],
        )
    }

    #[test]
    fn operation_div() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_DIVIDE, 1);
        machine_test(
            chunk,
            &[DataType::number(6.0), DataType::number(3.0)],
            &[DataType::number(2.0)],
            &[],
        )
    }

    #[test]
    fn operation_print() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_PRINT, 1);
        machine_test(
            chunk,
            &[DataType::text_from_str("abc")],
            &[],
            &["abc".to_string()],
        )
    }

    #[test]
    fn operation_constant() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_CONSTANT, 1);
        chunk.add_constant(DataType::number(2.0));
        let idx = chunk.add_constant(DataType::number(10.0));
        chunk.write_u8(idx as u8, 1);
        let mut machine = Machine::with(chunk, Rc::new(RefCell::new(DummyIO)));
        machine.run()?;
        assert_eq!(machine.stack_pop().unwrap().as_number(), Some(10.0));
        Ok(())
    }

    struct DummyIO;

    impl MachineIO for DummyIO {
        fn push_output(&mut self, _value: DataType) {
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
        stack_in: &[DataType],
        stack_out: &[DataType],
        buffer_out: &[String],
    ) -> MachineResult<()> {
        let probe_ref = Rc::new(RefCell::new(Probe::new()));
        let mut machine = Machine::with(chunk, probe_ref.clone());
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

        assert!(machine.stack.is_empty());
        Ok(())
    }
}
