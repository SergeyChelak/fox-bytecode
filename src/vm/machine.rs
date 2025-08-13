use crate::{
    chunk::{Chunk, Value},
    vm::{FetchError, Instruction},
};

const STACK_MAX_SIZE: usize = 256;

#[derive(Debug)]
pub enum MachineError {
    Compile(String),
    Runtime(String),
}

impl MachineError {
    pub fn runtime(message: &str) -> Self {
        Self::Runtime(message.to_string())
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
        'run_loop: loop {
            let ip = self.ip;
            let fetch_result = self.chunk.fetch(&mut self.ip);
            let instr = match fetch_result {
                Ok(instr) => instr,
                Err(FetchError::End) => break,
                Err(err) => return Err(MachineError::Runtime(format!("{err}"))),
            };
            println!("{}", self.chunk.disassemble_instruction(&instr, ip));
            match instr {
                Instruction::Constant(index) => {
                    let value = self.read_const(index)?;
                    self.stack_push(value)?;
                }
                Instruction::Add => self.do_binary(|a, b| a + b)?,
                Instruction::Subtract => self.do_binary(|a, b| a - b)?,
                Instruction::Multiply => self.do_binary(|a, b| a * b)?,
                Instruction::Divide => self.do_binary(|a, b| a / b)?,
                Instruction::Negate => {
                    let value = self.stack_pop()?;
                    self.stack_push(-value)?;
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

    fn do_binary(&mut self, operation: fn(Value, Value) -> Value) -> MachineResult<()> {
        let b = self.stack_pop()?;
        let a = self.stack_pop()?;
        let val = operation(a, b);
        self.stack_push(val)
    }

    fn read_const(&self, index: u8) -> MachineResult<Value> {
        let Some(value) = self.chunk.read_const(index) else {
            return Err(MachineError::runtime("Invalid constant index"));
        };
        Ok(value)
    }

    fn stack_reset(&mut self) {
        self.stack_top = 0;
    }

    fn stack_push(&mut self, value: Value) -> MachineResult<()> {
        if self.stack_top >= STACK_MAX_SIZE {
            return Err(MachineError::runtime("Stack overflow"));
        }
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
        Ok(())
    }

    fn stack_pop(&mut self) -> MachineResult<Value> {
        if self.stack_top == 0 {
            return Err(MachineError::runtime("Pop on empty stack"));
        }
        self.stack_top -= 1;
        Ok(self.stack[self.stack_top])
    }

    fn stack_trace(&self) {
        // 15 . 2 . 2 Stack tracing
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn operation_negate() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_opcode(crate::vm::OpCode::Negate, 1);
        let mut machine = Machine::with(chunk);
        machine.stack_push(10.0)?;
        machine.run()?;
        assert_eq!(machine.stack_pop().ok(), Some(-10.0));
        Ok(())
    }

    #[test]
    fn operation_add() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_opcode(crate::vm::OpCode::Add, 1);
        let mut machine = Machine::with(chunk);
        machine.stack_push(2.0)?;
        machine.stack_push(3.0)?;
        machine.run()?;
        assert_eq!(machine.stack_pop().ok(), Some(5.0));
        Ok(())
    }

    #[test]
    fn operation_sub() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_opcode(crate::vm::OpCode::Subtract, 1);
        let mut machine = Machine::with(chunk);
        machine.stack_push(2.0)?;
        machine.stack_push(3.0)?;
        machine.run()?;
        assert_eq!(machine.stack_pop().ok(), Some(-1.0));
        Ok(())
    }

    #[test]
    fn operation_mul() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_opcode(crate::vm::OpCode::Multiply, 1);
        let mut machine = Machine::with(chunk);
        machine.stack_push(2.0)?;
        machine.stack_push(3.0)?;
        machine.run()?;
        assert_eq!(machine.stack_pop().ok(), Some(6.0));
        Ok(())
    }

    #[test]
    fn operation_div() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_opcode(crate::vm::OpCode::Divide, 1);
        let mut machine = Machine::with(chunk);
        machine.stack_push(6.0)?;
        machine.stack_push(3.0)?;
        machine.run()?;
        assert_eq!(machine.stack_pop().ok(), Some(2.0));
        Ok(())
    }

    #[test]
    fn operation_constant() -> MachineResult<()> {
        let mut chunk = Chunk::new();
        chunk.write_opcode(crate::vm::OpCode::Constant, 1);
        chunk.add_constant(2.0);
        let idx = chunk.add_constant(10.0);
        chunk.write_u8(idx as u8, 1);
        let mut machine = Machine::with(chunk);
        machine.run()?;
        assert_eq!(machine.stack_pop().ok(), Some(10.0));
        Ok(())
    }
}
