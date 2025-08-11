use crate::{
    chunk::{Chunk, Value},
    vm::Instruction,
};

const STACK_MAX_SIZE: usize = 256;

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
            let Some(instr) = self.chunk.fetch(&mut self.ip) else {
                return Err(MachineError::runtime("Invalid instruction at {ip}"));
            };
            println!("{}", self.chunk.disassemble_instruction(&instr, ip));
            match instr {
                Instruction::Constant(index) => {
                    let value = self.read_const(index)?;
                    self.stack_push(value)?;
                }
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
