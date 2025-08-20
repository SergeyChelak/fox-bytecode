use std::fmt::Display;

use crate::{
    chunk::Chunk,
    data::{DataOperation, DataType, OperationError},
    vm::{FetchError, Instruction},
};

const STACK_MAX_SIZE: usize = 256;

#[derive(Debug)]
pub struct MachineError {
    text: String,
    line_number: Option<usize>,
}

impl Display for MachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = if let Some(num) = self.line_number {
            &format!("{num}")
        } else {
            "???"
        };
        write!(f, "[line {val}] {}", self.text)
    }
}

pub type MachineResult<T> = Result<T, MachineError>;

pub struct Machine {
    chunk: Chunk,
    ip: usize,
    stack: Vec<DataType>,
}

impl Machine {
    pub fn with(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::<DataType>::with_capacity(STACK_MAX_SIZE),
        }
    }
    pub fn run(&mut self) -> MachineResult<()> {
        let result = self.perform();
        if result.is_err() {
            self.stack_reset();
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
                Instruction::Return => {
                    let value = self.stack_pop()?;
                    println!("{value}");
                    break 'run_loop;
                }
            }
        }
        Ok(())
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

    fn stack_pop(&mut self) -> MachineResult<DataType> {
        let Some(value) = self.stack.pop() else {
            return Err(self.runtime_error("Pop on empty stack"));
        };
        Ok(value)
    }

    // fn stack_peek(&self, depth: usize) -> Option<DataType> {
    //     let len = self.stack.len();
    //     if len - 1 < depth {
    //         return None;
    //     }
    //     self.stack.get(len - 1 - depth).cloned()
    // }

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
        machine_test(chunk, &[DataType::number(10.0)], &[DataType::number(-10.0)])
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
        )?;
        machine_test(
            make_chunk(),
            &[DataType::number(3.0), DataType::number(2.0)],
            &[DataType::Bool(false)],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::Bool(false), DataType::Bool(false)],
            &[DataType::Bool(true)],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::Nil, DataType::Nil],
            &[DataType::Bool(true)],
        )?;
        machine_test(
            make_chunk(),
            &[DataType::Bool(false), DataType::Nil],
            &[DataType::Bool(false)],
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
        )?;
        machine_test(
            make_chunk(),
            &[DataType::number(1.0), DataType::number(2.0)],
            &[DataType::Bool(false)],
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
        )?;
        machine_test(
            make_chunk(),
            &[DataType::number(1.0), DataType::number(2.0)],
            &[DataType::Bool(true)],
        )?;

        let res = machine_test(
            make_chunk(),
            &[DataType::number(1.0), DataType::Bool(true)],
            &[],
        );
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn operation_nil() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_NIL, 1);
        machine_test(chunk, &[], &[DataType::Nil])
    }

    #[test]
    fn operation_true() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_TRUE, 1);
        machine_test(chunk, &[], &[DataType::Bool(true)])
    }

    #[test]
    fn operation_false() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_FALSE, 1);
        machine_test(chunk, &[], &[DataType::Bool(false)])
    }

    #[test]
    fn operation_add() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_ADD, 1);
        machine_test(
            chunk,
            &[DataType::number(2.0), DataType::number(3.0)],
            &[DataType::number(5.0)],
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
        )
    }

    #[test]
    fn operation_constant() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_CONSTANT, 1);
        chunk.add_constant(DataType::number(2.0));
        let idx = chunk.add_constant(DataType::number(10.0));
        chunk.write_u8(idx as u8, 1);
        let mut machine = Machine::with(chunk);
        machine.run()?;
        assert_eq!(machine.stack_pop().unwrap().as_number(), Some(10.0));
        Ok(())
    }

    // #[test]
    // fn stack_peek() -> MachineResult<()> {
    //     let chunk = Chunk::new();
    //     let mut machine = Machine::with(chunk);
    //     machine.stack_push(DataType::Number(1.0))?;
    //     machine.stack_push(DataType::Number(2.0))?;

    //     assert_eq!(machine.stack_peek(0).unwrap(), DataType::Number(2.0));
    //     assert_eq!(machine.stack_peek(1).unwrap(), DataType::Number(1.0));

    //     assert!(machine.stack_peek(3).is_none());
    //     Ok(())
    // }

    fn machine_test(
        chunk: Chunk,
        stack_in: &[DataType],
        stack_out: &[DataType],
    ) -> MachineResult<()> {
        let mut machine = Machine::with(chunk);
        for v_in in stack_in {
            machine.stack_push(v_in.clone())?;
        }
        machine.run()?;
        for v_out in stack_out.iter().rev() {
            let val = machine.stack_pop()?;
            assert_eq!(val, *v_out);
        }
        assert!(machine.stack.is_empty());
        Ok(())
    }
}
