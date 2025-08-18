use std::fmt::Display;

use crate::{
    chunk::Chunk,
    value::{Double, Value},
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
    stack: [Value; STACK_MAX_SIZE],
    stack_top: usize,
}

impl Machine {
    pub fn with(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: [Value::default(); STACK_MAX_SIZE],
            stack_top: 0,
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
                Instruction::Equal => {
                    let b = self.stack_pop()?;
                    let a = self.stack_pop()?;
                    self.stack_push(Value::Bool(a == b))?;
                }
                Instruction::Greater => self.do_binary(|a, b| Value::Bool(a > b))?,
                Instruction::Less => self.do_binary(|a, b| Value::Bool(a < b))?,
                Instruction::Nil => self.stack_push(Value::Nil)?,
                Instruction::True => self.stack_push(Value::Bool(true))?,
                Instruction::False => self.stack_push(Value::Bool(false))?,
                Instruction::Add => self.do_binary(|a, b| Value::number(a + b))?,
                Instruction::Subtract => self.do_binary(|a, b| Value::number(a - b))?,
                Instruction::Multiply => self.do_binary(|a, b| Value::number(a * b))?,
                Instruction::Divide => self.do_binary(|a, b| Value::number(a / b))?,
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
                Instruction::Return => {
                    let value = self.stack_pop()?;
                    println!("{value}");
                    break 'run_loop;
                }
            }
        }
        Ok(())
    }

    fn do_binary(&mut self, operation: fn(Double, Double) -> Value) -> MachineResult<()> {
        let b = self.stack_pop()?;
        let a = self.stack_pop()?;
        let (Some(a), Some(b)) = (a.as_number(), b.as_number()) else {
            return Err(self.runtime_error("Operands must be numbers"));
        };
        let val = operation(a, b);
        self.stack_push(val)
    }

    fn read_const(&self, index: u8) -> MachineResult<Value> {
        let Some(value) = self.chunk.read_const(index) else {
            return Err(self.runtime_error("Invalid constant index"));
        };
        Ok(value)
    }

    fn stack_reset(&mut self) {
        self.stack_top = 0;
    }

    fn stack_push(&mut self, value: Value) -> MachineResult<()> {
        if self.stack_top >= STACK_MAX_SIZE {
            return Err(self.runtime_error("Stack overflow"));
        }
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
        Ok(())
    }

    fn stack_pop(&mut self) -> MachineResult<Value> {
        if self.stack_top == 0 {
            return Err(self.runtime_error("Pop on empty stack"));
        }
        self.stack_top -= 1;
        Ok(self.stack[self.stack_top])
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
        machine_test(chunk, &[Value::number(10.0)], &[Value::number(-10.0)])
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
        )?;
        machine_test(
            make_chunk(),
            &[Value::number(3.0), Value::number(2.0)],
            &[Value::Bool(false)],
        )?;
        machine_test(
            make_chunk(),
            &[Value::Bool(false), Value::Bool(false)],
            &[Value::Bool(true)],
        )?;
        machine_test(
            make_chunk(),
            &[Value::Nil, Value::Nil],
            &[Value::Bool(true)],
        )?;
        machine_test(
            make_chunk(),
            &[Value::Bool(false), Value::Nil],
            &[Value::Bool(false)],
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
        )?;
        machine_test(
            make_chunk(),
            &[Value::number(1.0), Value::number(2.0)],
            &[Value::Bool(false)],
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
        )?;
        machine_test(
            make_chunk(),
            &[Value::number(1.0), Value::number(2.0)],
            &[Value::Bool(true)],
        )?;

        let res = machine_test(make_chunk(), &[Value::number(1.0), Value::Bool(true)], &[]);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn operation_nil() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_NIL, 1);
        machine_test(chunk, &[], &[Value::Nil])
    }

    #[test]
    fn operation_true() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_TRUE, 1);
        machine_test(chunk, &[], &[Value::Bool(true)])
    }

    #[test]
    fn operation_false() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_FALSE, 1);
        machine_test(chunk, &[], &[Value::Bool(false)])
    }

    #[test]
    fn operation_add() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_ADD, 1);
        machine_test(
            chunk,
            &[Value::number(2.0), Value::number(3.0)],
            &[Value::number(5.0)],
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
        )
    }

    #[test]
    fn operation_constant() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_u8(OPCODE_CONSTANT, 1);
        chunk.add_constant(Value::number(2.0));
        let idx = chunk.add_constant(Value::number(10.0));
        chunk.write_u8(idx as u8, 1);
        let mut machine = Machine::with(chunk);
        machine.run()?;
        assert_eq!(machine.stack_pop().unwrap().as_number(), Some(10.0));
        Ok(())
    }

    fn machine_test(chunk: Chunk, stack_in: &[Value], stack_out: &[Value]) -> MachineResult<()> {
        let mut machine = Machine::with(chunk);
        for v_in in stack_in {
            machine.stack_push(*v_in)?;
        }
        machine.run()?;
        for v_out in stack_out.iter().rev() {
            let val = machine.stack_pop()?;
            assert_eq!(val, *v_out);
        }
        assert_eq!(machine.stack_top, 0);
        Ok(())
    }
}
