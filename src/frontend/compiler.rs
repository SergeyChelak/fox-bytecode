use crate::{Chunk, FetchResult, Func, FuncType, Instruction, UINT8_COUNT, Value, frontend::Token};

pub const MAX_SCOPE_SIZE: usize = UINT8_COUNT;

pub struct Compiler {
    func: Box<Func>,
    func_type: FuncType,
    locals: Vec<Local>,
    depth: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            func: Default::default(),
            func_type: FuncType::Script,
            locals: Vec::new(),
            // locals: vec![Local::reserved()],
            depth: Default::default(),
        }
    }
}

/// Shorthands
impl Compiler {
    pub fn chunk(&self) -> &Chunk {
        self.func.chunk()
    }

    pub fn chunk_mut(&mut self) -> &mut Chunk {
        self.func.chunk_mut()
    }

    pub fn function(self) -> Func {
        *self.func
    }

    pub fn chunk_position(&self) -> usize {
        self.func.chunk().size()
    }
}

/// Code generation functions
impl Compiler {
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.chunk_mut().add_constant(value)
    }

    pub fn emit_instruction_at_line(&mut self, instruction: &Instruction, line: usize) -> usize {
        let start = self.chunk_position();
        let bytes: Vec<u8> = instruction.as_vec();
        for byte in bytes.into_iter() {
            self.chunk_mut().write_u8(byte, line);
        }
        start
    }

    pub fn patch_instruction(&mut self, instruction: &Instruction, offset: usize) {
        let bytes: Vec<u8> = instruction.as_vec();
        for (idx, byte) in bytes.into_iter().enumerate() {
            self.chunk_mut().patch_u8(byte, offset + idx);
        }
    }

    pub fn fetch_instruction(&self, offset: usize) -> (FetchResult<Instruction>, usize) {
        let mut idx = offset;
        let res = self.chunk().fetch(&mut idx);
        let size = idx - offset;
        (res, size)
    }
}

/// Scope management functions
impl Compiler {
    pub fn begin_scope(&mut self) {
        self.depth += 1;
    }

    pub fn end_scope(&mut self, line: usize) {
        self.depth -= 1;
        while self.is_last_out_of_scope() {
            self.emit_instruction_at_line(&Instruction::Pop, line);
            self.locals.pop();
        }
    }

    pub fn is_global_scope(&self) -> bool {
        self.depth == 0
    }

    pub fn is_local_scope(&self) -> bool {
        self.depth > 0
    }

    pub fn has_capacity(&self) -> bool {
        self.locals.len() < MAX_SCOPE_SIZE
    }

    pub fn push_local(&mut self, local: Local) {
        self.locals.push(local);
    }

    pub fn has_declared_variable(&self, token: &Token) -> bool {
        for local in self.locals.iter().rev() {
            let Some(depth) = local.depth else {
                break;
            };
            if depth < self.depth {
                break;
            }
            if local.name == token.text {
                return true;
            }
        }
        false
    }

    pub fn resolve_local(&self, token: &Token) -> Option<LocalVariableInfo> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == token.text {
                let info = LocalVariableInfo {
                    index: i as u8,
                    depth: local.depth,
                };
                return Some(info);
            }
        }
        None
    }

    fn is_last_out_of_scope(&mut self) -> bool {
        let Some(depth) = self.locals.last().and_then(|local| local.depth) else {
            return false;
        };
        depth > self.depth
    }

    pub fn mark_initialized(&mut self) {
        let Some(local) = self.locals.last_mut() else {
            panic!();
        };
        local.depth = Some(self.depth);
    }
}

pub struct LocalVariableInfo {
    pub index: u8,
    pub depth: Option<usize>,
}

pub struct Local {
    name: String,
    depth: Option<usize>,
}

impl Local {
    pub fn with_name(name: String) -> Self {
        Self { name, depth: None }
    }

    fn reserved() -> Self {
        Self {
            name: "".to_string(),
            depth: Some(0),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn patch_instruction() {
        let mut compiler = Compiler::new();
        compiler.emit_instruction_at_line(&Instruction::Add, 0);
        let emit_addr = compiler.emit_instruction_at_line(&Instruction::Constant(1), 0);
        compiler.emit_instruction_at_line(&Instruction::Subtract, 0);
        compiler.emit_instruction_at_line(&Instruction::Return, 0);
        compiler.patch_instruction(&Instruction::Constant(2), emit_addr);

        let chunk = compiler.chunk();
        let mut offset = 0;
        let expected = &[
            Instruction::Add,
            Instruction::Constant(2),
            Instruction::Subtract,
            Instruction::Return,
        ];
        let mut exp_idx = 0;
        while let Ok(instr) = chunk.fetch(&mut offset) {
            assert_eq!(instr, expected[exp_idx]);
            exp_idx += 1;
        }
    }
}
