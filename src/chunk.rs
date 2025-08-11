use crate::vm::{Instruction, OpCode};

pub type Value = f32;

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    line: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Default::default(),
            constants: Default::default(),
            line: Default::default(),
        }
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) {
        self.write_u8(opcode as u8, line);
    }

    pub fn write_u8(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.line.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn fetch(&mut self, offset: &mut usize) -> Option<Instruction> {
        Instruction::fetch(&self.code, offset)
    }

    pub fn disassemble(&mut self) -> String {
        let mut output = Vec::new();
        let mut offset = 0;
        loop {
            let start = offset;
            let Some(instr) = self.fetch(&mut offset) else {
                break;
            };
            let info = self.disassemble_instruction(&instr, start);
            output.push(info);
        }
        output.join("\n")
    }

    fn disassemble_instruction(&mut self, instr: &Instruction, offset: usize) -> String {
        let main = format!("{instr}");
        let value = match instr {
            Instruction::Constant(idx) => format!("{main}\t{}", self.constants[*idx as usize]),
            _ => main,
        };
        if offset > 0 && self.line[offset] == self.line[offset - 1] {
            format!("   | {value}")
        } else {
            format!("{:4} {value}", self.line[offset])
        }
    }
}
