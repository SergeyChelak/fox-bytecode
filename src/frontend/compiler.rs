use crate::{
    Chunk, FetchResult, Func, FuncType, Instruction, UINT8_COUNT, UpvalueData, Value,
    frontend::Token,
};

pub const MAX_SCOPE_SIZE: usize = UINT8_COUNT;
type UpvalueDataArray = [UpvalueData; UINT8_COUNT];

pub struct Compiler {
    func: Box<Func>,
    func_type: FuncType,
    locals: Vec<Local>,
    depth: usize,
    upvalues: UpvalueDataArray,
    pub(crate) enclosing: Option<Box<Compiler>>,
}

impl Compiler {
    pub fn with(func_type: FuncType, enclosing: Option<Box<Compiler>>) -> Self {
        Self {
            func: Default::default(),
            func_type,
            locals: vec![Local::reserved()],
            depth: Default::default(),
            upvalues: [Default::default(); UINT8_COUNT],
            enclosing,
        }
    }

    pub fn assign_name(&mut self, name: &str) {
        self.func.name = Some(name.to_string());
    }

    pub fn func_type(&self) -> &FuncType {
        &self.func_type
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

    pub fn function_consumed(self) -> Func {
        *self.func
    }

    pub fn consume_closure_data(self) -> (Func, UpvalueDataArray) {
        (*self.func, self.upvalues)
    }

    pub fn function(&self) -> &Func {
        self.func.as_ref()
    }

    pub fn function_mut(&mut self) -> &mut Func {
        self.func.as_mut()
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
        self.emit_buffer(&bytes, line);
        start
    }

    pub fn emit_buffer(&mut self, buffer: &[u8], line: usize) {
        self.chunk_mut().write_buffer(buffer, line)
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
            let is_captured = self.locals.last().map(|x| x.is_captured).unwrap_or(false);
            if is_captured {
                self.emit_instruction_at_line(&Instruction::CloseUpvalue, line);
            } else {
                self.emit_instruction_at_line(&Instruction::Pop, line);
            }
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

    pub fn resolve_local(&self, token: &Token) -> Option<LocalData> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == token.text {
                let info = LocalData {
                    index: i as u8,
                    depth: local.depth,
                };
                return Some(info);
            }
        }
        None
    }

    pub fn resolve_upvalue(&mut self, token: &Token) -> UpvalueResolve {
        let Some(enclosing) = self.enclosing.as_mut() else {
            return UpvalueResolve::NotFound;
        };

        if let Some(local) = enclosing.resolve_local(token) {
            let index = local.index;
            enclosing
                .locals
                .get_mut(index as usize)
                .map(|data| data.is_captured = true)
                .expect("Failed to update captured flag");
            return self.add_upvalue(index, true).into();
        }

        match enclosing.resolve_upvalue(token) {
            UpvalueResolve::Index(upvalue) => self.add_upvalue(upvalue, false).into(),
            any => any,
        }
    }

    fn add_upvalue(&mut self, index: u8, is_local: bool) -> Result<usize, &'static str> {
        let count = self.func.upvalue_count;
        let data = UpvalueData { index, is_local };

        if let Some((i, _)) = self
            .upvalues
            .iter()
            .take(count)
            .enumerate()
            .find(|(_, x)| *x == &data)
        {
            return Ok(i);
        }
        if count == UINT8_COUNT {
            return Err("Too many closure variables in function");
        }
        self.upvalues[count] = data;
        self.func.upvalue_count = count + 1;
        Ok(count)
    }

    fn is_last_out_of_scope(&mut self) -> bool {
        let Some(depth) = self.locals.last().and_then(|local| local.depth) else {
            return false;
        };
        depth > self.depth
    }

    pub fn mark_initialized(&mut self) {
        if self.depth == 0 {
            return;
        }
        let Some(local) = self.locals.last_mut() else {
            panic!();
        };
        local.depth = Some(self.depth);
    }
}

pub struct LocalData {
    pub index: u8,
    pub depth: Option<usize>,
}

pub struct Local {
    name: String,
    depth: Option<usize>,
    is_captured: bool,
}

impl Local {
    pub fn with_name(name: String) -> Self {
        Self {
            name,
            depth: None,
            is_captured: false,
        }
    }

    fn reserved() -> Self {
        Self {
            name: "".to_string(),
            depth: Some(0),
            is_captured: false,
        }
    }
}

pub enum UpvalueResolve {
    NotFound,
    Index(u8),
    Error(&'static str),
}

impl From<Result<usize, &'static str>> for UpvalueResolve {
    fn from(value: Result<usize, &'static str>) -> Self {
        match value {
            Ok(index) => UpvalueResolve::Index(index as u8),
            Err(msg) => UpvalueResolve::Error(msg),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn patch_instruction() {
        let mut compiler = Compiler::with(FuncType::Script, None);
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
