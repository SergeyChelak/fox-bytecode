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
        self.parse_precedence(Precedence::Assignment);
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

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let t_type = self.prev_token_type();
        let Some(prefix_rule) = self.get_rule(t_type).prefix else {
            self.error("Expect expression");
            return;
        };
        prefix_rule(self);

        while precedence.le(&self.get_rule(self.cur_token_type()).precedence) {
            self.advance();
            let infix_rule = self
                .get_rule(self.prev_token_type())
                .infix
                .expect("Infix is none");
            infix_rule(self);
        }
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
        // TODO: made according to the book, looks bad...
        let operator_type = self.prev_token_type();
        // Compile the operand
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction
        if matches!(operator_type, TokenType::Minus) {
            self.emit_instruction(Instruction::Negate)
        }
    }

    fn binary(&mut self) {
        let operator_type = self.prev_token_type();
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.increased());

        let inst = match operator_type {
            TokenType::Plus => Instruction::Add,
            TokenType::Minus => Instruction::Subtract,
            TokenType::Star => Instruction::Multiply,
            TokenType::Slash => Instruction::Divide,
            x => unreachable!("Unexpected binary operator {x:?}"),
        };
        self.emit_instruction(inst);
    }

    fn prev_token_type(&self) -> TokenType {
        self.previous
            .as_ref()
            .map(|t| t.t_type)
            .expect("Bug: previous token is none")
    }

    fn cur_token_type(&self) -> TokenType {
        self.current
            .as_ref()
            .map(|t| t.t_type)
            .expect("Bug: previous token is none")
    }

    fn get_rule(&self, t_type: TokenType) -> ParseRule {
        use TokenType::*;
        match t_type {
            LeftParenthesis => ParseRule::new(Some(Self::grouping), None, Precedence::None),
            Minus => ParseRule::new(Some(Self::unary), Some(Self::binary), Precedence::Term),
            Plus => ParseRule::new(None, Some(Self::binary), Precedence::Term),
            Slash | Star => ParseRule::new(None, Some(Self::binary), Precedence::Factor),
            Number => ParseRule::new(Some(Self::number), None, Precedence::None),
            _ => Default::default(),
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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn increased(&self) -> Self {
        use Precedence::*;
        match self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => Primary, //unreachable!("undefined behavior by the book"),
        }
    }

    fn le(&self, other: &Self) -> bool {
        *self as u8 <= *other as u8
    }
}

type ParseFn = fn(&mut Parser);

struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl ParseRule {
    fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

impl Default for ParseRule {
    fn default() -> Self {
        Self {
            precedence: Precedence::None,
            prefix: Default::default(),
            infix: Default::default(),
        }
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;
}

#[cfg(test)]
mod test_precedence {
    use super::*;

    #[test]
    fn increase_less_equal() {
        use Precedence::*;
        let priorities = [
            None, Assignment, Or, And, Equality, Comparison, Term, Factor, Unary, Call, Primary,
        ];

        for (i, item) in priorities.iter().enumerate() {
            let next = item.increased();
            assert!(item.le(&item));
            assert!(item.le(&next));
            let next_val = priorities.get(i + 1).unwrap_or(&Precedence::Primary);
            assert_eq!(next, *next_val);
        }
    }
}
