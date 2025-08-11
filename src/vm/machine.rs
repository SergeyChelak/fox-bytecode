use crate::{chunk::Chunk, vm::Instruction};

pub enum MachineError {
    Compile(String),
    Runtime(String),
}

pub type MachineResult<T> = Result<T, MachineError>;

pub struct Machine {
    chunk: Chunk,
    ip: usize,
}

impl Machine {
    pub fn with(chunk: Chunk) -> Self {
        Self { chunk, ip: 0 }
    }

    pub fn run(&mut self) -> MachineResult<()> {
        loop {
            let Some(instr) = self.chunk.fetch(&mut self.ip) else {
                panic!();
            };
            match instr {
                Instruction::Constant(_) => todo!(),
                Instruction::Return => break,
            }
        }
        Ok(())
    }
}
