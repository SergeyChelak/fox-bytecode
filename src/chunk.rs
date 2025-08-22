use crate::{
    data::DataType,
    vm::{FetchResult, Instruction},
};

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<DataType>,
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

    pub fn write_u8(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.line.push(line);
    }

    pub fn add_constant(&mut self, value: DataType) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn read_const(&self, idx: u8) -> Option<DataType> {
        self.constants.get(idx as usize).cloned()
    }

    pub fn fetch(&self, offset: &mut usize) -> FetchResult<Instruction> {
        Instruction::fetch(&self.code, offset)
    }

    pub fn line_number(&self, idx: usize) -> Option<usize> {
        self.line.get(idx).cloned()
    }
}
