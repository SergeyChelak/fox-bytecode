use crate::{
    chunk::{Chunk, Value},
    compiler::{
        scanner::Scanner,
        token::{Token, TokenType},
    },
    utils::ErrorInfo,
    vm::Instruction,
};

pub struct Parser {
    current: Option<Token>,
    previous: Option<Token>,
    scanner: Scanner,
    panic_mode: bool,
    chunk: Chunk,
    errors: Vec<ErrorInfo>,
}

impl Parser {
    pub fn with(scanner: Scanner) -> Self {
        Self {
            current: Default::default(),
            previous: Default::default(),
            scanner,
            panic_mode: false,
            chunk: Chunk::new(),
            errors: Default::default(),
        }
    }

    pub fn compile(&mut self) -> Result<Chunk, Vec<ErrorInfo>> {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression");
        self.end_compiler();
        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        todo!()
    }

    fn advance(&mut self) {
        self.previous = self.current.take();
        loop {
            let token = self.scanner.scan_token();
            let is_err = token.is_err();
            self.current = Some(token);
            if is_err {
                self.error_at_current("");
            } else {
                break;
            }
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.push_error_info(self.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.push_error_info(self.previous.clone(), message);
    }

    fn push_error_info(&mut self, elem: Option<Token>, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        let info = ErrorInfo::with(elem, message);
        self.errors.push(info);
    }

    fn expression(&mut self) {
        todo!()
    }

    fn consume<T: AsRef<str>>(&mut self, t_type: TokenType, message: T) {
        if let Some(cur) = &self.current
            && cur.t_type == t_type
        {
            self.advance();
            return;
        };

        self.error_at_current(message.as_ref());
    }

    fn chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunk
    }

    fn end_compiler(&mut self) {
        self.emit_return();
    }

    fn emit_return(&mut self) {
        self.emit_instruction(Instruction::Return)
    }

    fn number(&mut self) {
        // I don't like this approach
        // according to strtod it returns 0.0 as fallback
        let value = self
            .previous
            .as_ref()
            .and_then(|token| token.text.parse::<Value>().ok())
            .unwrap_or_default();
        self.emit_constant(value)
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expect ')' after expression");
    }

    fn unary(&mut self) {
        // TODO: made according to the book, don't think this is acceptable
        let operator_type = self.previous.as_ref().map(|t| t.t_type);
        self.expression();
        if matches!(operator_type, Some(TokenType::Minus)) {
            self.emit_instruction(Instruction::Negate)
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let idx = self.make_constant(value);
        self.emit_instruction(Instruction::Constant(idx));
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let chunk = self.chunk_mut();
        let idx = chunk.add_constant(value);
        if idx > u8::MAX as usize {
            self.error("Too many constants in one chunk");
            // don't think it's a good decision
            // but this index seems doesn't reachable
            return 0;
        }

        idx as u8
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        let line = self
            .previous
            .as_ref()
            .map(|x| x.position.line)
            .unwrap_or_default();
        let bytes: Vec<u8> = instruction.into();
        for byte in bytes.into_iter() {
            self.chunk_mut().write_u8(byte, line);
        }
    }
}
