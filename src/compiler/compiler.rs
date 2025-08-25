use crate::{
    Chunk, ErrorCollector, Func, FuncType, Instruction, Value,
    compiler::Token,
    utils::{Shared, jump_to_bytes},
};

const MAX_SCOPE_SIZE: usize = 256;

pub struct LocalVariableInfo {
    pub index: u8,
    pub depth: Option<usize>,
}

pub struct Compiler {
    func: Box<Func>,
    func_type: FuncType,
    locals: Vec<Local>,
    depth: usize,
    error_collector: Shared<ErrorCollector>,
}

impl Compiler {
    pub fn new(error_collector: Shared<ErrorCollector>) -> Self {
        Self {
            func: Default::default(),
            func_type: FuncType::Script,
            locals: Default::default(),
            depth: Default::default(),
            error_collector,
        }
    }
}

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

impl Compiler {
    fn make_constant(&mut self, value: Value) -> u8 {
        let idx = self.chunk_mut().add_constant(value);
        if idx > u8::MAX as usize {
            self.error_collector
                .borrow_mut()
                .error("Too many constants in one chunk");
            // don't think it's a good decision
            // but this index seems doesn't reachable
            return 0;
        }
        idx as u8
    }

    fn emit_constant(&mut self, value: Value, line: usize) {
        let idx = self.make_constant(value);
        self.emit_instruction_at_line(&Instruction::Constant(idx), line);
    }

    fn emit_loop(&mut self, loop_start: usize, line: usize) {
        let instr = Instruction::Loop(0x0, 0x0);
        let size = instr.size();
        let offset = self.chunk_position() - loop_start + size;
        if offset > u16::MAX as usize {
            self.error_collector
                .borrow_mut()
                .error("Jump size is too large");
        }
        let (f, s) = jump_to_bytes(offset);
        self.emit_instruction_at_line(&Instruction::Loop(f, s), line);
    }

    fn emit_return(&mut self, line: usize) {
        self.emit_instruction_at_line(&Instruction::Return, line);
    }

    fn emit_instruction_at_line(&mut self, instruction: &Instruction, line: usize) -> usize {
        let start = self.chunk_position();
        let bytes: Vec<u8> = instruction.as_vec();
        for byte in bytes.into_iter() {
            self.chunk_mut().write_u8(byte, line);
        }
        start
    }

    fn patch_instruction(&mut self, instruction: &Instruction, offset: usize) {
        let bytes: Vec<u8> = instruction.as_vec();
        for (idx, byte) in bytes.into_iter().enumerate() {
            self.chunk_mut().patch_u8(byte, offset + idx);
        }
    }

    fn patch_jump(&mut self, offset: usize) {
        let (fetch_result, size) = {
            let mut idx = offset;
            let res = self.chunk().fetch(&mut idx);
            let size = idx - offset;
            (res, size)
        };

        let jump = self.chunk_position() - offset - size;
        if jump > u16::MAX as usize {
            self.error_collector
                .borrow_mut()
                .error("Too much code to jump over");
        }
        let (first, second) = jump_to_bytes(jump);
        let instr = match fetch_result {
            Ok(Instruction::JumpIfFalse(_, _)) => Instruction::JumpIfFalse(first, second),
            Ok(Instruction::Jump(_, _)) => Instruction::Jump(first, second),
            Err(err) => {
                self.error_collector
                    .borrow_mut()
                    .error(&format!("Bug: {err}"));
                return;
            }
            _ => {
                self.error_collector
                    .borrow_mut()
                    .error("Bug: Attempt to patch non-jump instruction in 'path_jump' function");
                return;
            }
        };
        self.patch_instruction(&instr, offset);
    }
}

impl Compiler {
    pub fn begin_scope(&mut self) {
        self.depth += 1;
    }

    pub fn end_scope(&mut self) -> usize {
        self.depth -= 1;
        let mut pop_count = 0;
        while self.is_last_out_of_scope() {
            pop_count += 1;
            self.locals.pop();
        }
        pop_count
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

    pub fn push(&mut self, local: Local) {
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

pub struct Local {
    name: String,
    depth: Option<usize>,
}

impl Local {
    pub fn with_name(name: String) -> Self {
        Self { name, depth: None }
    }
}
