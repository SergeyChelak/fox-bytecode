use crate::{Chunk, Func, FuncType, compiler::Token};

const MAX_SCOPE_SIZE: usize = 256;

pub struct LocalVariableInfo {
    pub index: u8,
    pub depth: Option<usize>,
}

// #[derive(Default)]
pub struct Compiler {
    func: Box<Func>,
    func_type: FuncType,
    locals: Vec<Local>,
    depth: usize,
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            func: Default::default(),
            func_type: FuncType::Script,
            locals: Default::default(),
            depth: Default::default(),
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
