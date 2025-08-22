use crate::compiler::Token;

const MAX_SCOPE_SIZE: usize = 256;

// I think the 'Compiler' is strange name for scope manager
#[derive(Default)]
pub struct Scope {
    locals: Vec<Local>,
    depth: usize,
}

impl Scope {
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

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn is_global(&self) -> bool {
        self.depth == 0
    }

    pub fn is_local(&self) -> bool {
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
            if local.name.text == token.text {
                return true;
            }
        }
        false
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
    name: Token,
    // TODO: remove pub!
    pub depth: Option<usize>,
}

impl Local {
    pub fn with_token(token: Token) -> Self {
        Self {
            name: token,
            depth: None,
        }
    }
}
