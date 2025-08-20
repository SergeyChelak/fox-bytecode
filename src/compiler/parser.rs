use crate::{
    chunk::Chunk,
    compiler::{
        scanner::TokenSource,
        token::{Token, TokenType},
    },
    data::DataType,
    utils::ErrorInfo,
    vm::Instruction,
};

pub struct Parser {
    current: Option<Token>,
    previous: Option<Token>,
    scanner: Box<dyn TokenSource>,
    panic_mode: bool,
    chunk: Chunk,
    errors: Vec<ErrorInfo>,
}

impl Parser {
    pub fn with(scanner: Box<dyn TokenSource>) -> Self {
        Self {
            current: Default::default(),
            previous: Default::default(),
            scanner,
            panic_mode: false,
            chunk: Chunk::new(),
            errors: Default::default(),
        }
    }

    pub fn compile(&mut self) -> Result<(), Vec<ErrorInfo>> {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression");
        self.end_compiler();
        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        Ok(())
        // todo!()
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

    fn end_compiler(&mut self) {
        self.emit_return();
    }

    fn emit_return(&mut self) {
        self.emit_instruction(&Instruction::Return)
    }

    fn number(&mut self) {
        // I don't like this approach
        // according to strtod it returns 0.0 as fallback
        let value = self
            .previous
            .as_ref()
            .and_then(|token| DataType::number_from(&token.text).ok())
            .unwrap_or(DataType::number(0.0));
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
        match operator_type {
            TokenType::Minus => self.emit_instruction(&Instruction::Negate),
            TokenType::Bang => self.emit_instruction(&Instruction::Not),
            _ => unreachable!("unary"),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.prev_token_type();
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.increased());

        let array: &[Instruction] = match operator_type {
            TokenType::BangEqual => &[Instruction::Equal, Instruction::Not],
            TokenType::EqualEqual => &[Instruction::Equal],
            TokenType::Greater => &[Instruction::Greater],
            TokenType::GreaterEqual => &[Instruction::Less, Instruction::Not],
            TokenType::Less => &[Instruction::Less],
            TokenType::LessEqual => &[Instruction::Greater, Instruction::Not],
            TokenType::Plus => &[Instruction::Add],
            TokenType::Minus => &[Instruction::Subtract],
            TokenType::Star => &[Instruction::Multiply],
            TokenType::Slash => &[Instruction::Divide],
            x => unreachable!("Unexpected binary operator {x:?}"),
        };
        self.emit_instructions(array);
    }

    fn literal(&mut self) {
        match self.prev_token_type() {
            TokenType::False => self.emit_instruction(&Instruction::False),
            TokenType::True => self.emit_instruction(&Instruction::True),
            TokenType::Nil => self.emit_instruction(&Instruction::Nil),
            _ => unreachable!("literal"),
        }
    }

    fn string(&mut self) {
        let text = self
            .previous
            .as_ref()
            .map(|t| &t.text)
            .map(|s| &s[1..s.len() - 1])
            .expect("Bug: failed to extract string value");
        self.emit_constant(DataType::str_text(text));
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
            Nil | False | True => ParseRule::new(Some(Self::literal), None, Precedence::None),
            Bang => ParseRule::new(Some(Self::unary), None, Precedence::None),
            EqualEqual | BangEqual => {
                ParseRule::new(None, Some(Self::binary), Precedence::Equality)
            }
            Greater | GreaterEqual | Less | LessEqual => {
                ParseRule::new(None, Some(Self::binary), Precedence::Comparison)
            }
            TokenType::String => ParseRule::new(Some(Self::string), None, Precedence::None),
            _ => Default::default(),
        }
    }

    fn emit_constant(&mut self, value: DataType) {
        let idx = self.make_constant(value);
        self.emit_instruction(&Instruction::Constant(idx));
    }

    fn make_constant(&mut self, value: DataType) -> u8 {
        let idx = self.chunk.add_constant(value);
        if idx > u8::MAX as usize {
            self.error("Too many constants in one chunk");
            // don't think it's a good decision
            // but this index seems doesn't reachable
            return 0;
        }

        idx as u8
    }

    fn emit_instruction(&mut self, instruction: &Instruction) {
        let line = self
            .previous
            .as_ref()
            .map(|x| x.position.line)
            .unwrap_or_default();
        let bytes: Vec<u8> = instruction.as_vec();
        for byte in bytes.into_iter() {
            self.chunk.write_u8(byte, line);
        }
    }

    fn emit_instructions(&mut self, instruction: &[Instruction]) {
        instruction
            .iter()
            .for_each(|inst| self.emit_instruction(inst));
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
    use crate::utils::CodePosition;

    #[test]
    fn emit_unary_chunk() {
        let input = vec![Token::minus(), Token::number("12.345")];
        let expectation = Expectation {
            constants: vec![DataType::number(12.345)],
            instructions: vec![Instruction::Constant(0), Instruction::Negate],
        };
        state_expectation_test(input, expectation);
    }

    #[test]
    fn emit_binary_chunk() {
        let data = [
            (
                Token::make(TokenType::BangEqual, "!="),
                vec![Instruction::Equal, Instruction::Not],
            ),
            (
                Token::make(TokenType::EqualEqual, "=="),
                vec![Instruction::Equal],
            ),
            (
                Token::make(TokenType::Greater, ">"),
                vec![Instruction::Greater],
            ),
            (
                Token::make(TokenType::GreaterEqual, ">="),
                vec![Instruction::Less, Instruction::Not],
            ),
            (Token::make(TokenType::Less, "<"), vec![Instruction::Less]),
            (
                Token::make(TokenType::LessEqual, "<="),
                vec![Instruction::Greater, Instruction::Not],
            ),
            (Token::minus(), vec![Instruction::Subtract]),
            (Token::plus(), vec![Instruction::Add]),
            (Token::multiply(), vec![Instruction::Multiply]),
            (Token::divide(), vec![Instruction::Divide]),
        ];
        for (token, expected_instr) in data {
            let input = vec![Token::number("3"), token, Token::number("5.0")];
            let mut instructions = vec![Instruction::Constant(0), Instruction::Constant(1)];

            for exp_instr in expected_instr {
                instructions.push(exp_instr.clone());
            }
            let expectation = Expectation {
                constants: vec![DataType::number(3.0), DataType::number(5.0)],
                instructions,
            };
            state_expectation_test(input, expectation);
        }
    }

    #[test]
    fn emit_literal_chunk() {
        let tokens = vec![
            (Token::make(TokenType::False, "false"), Instruction::False),
            (Token::make(TokenType::True, "true"), Instruction::True),
            (Token::make(TokenType::Nil, "nil"), Instruction::Nil),
        ];
        for (token, emitted) in tokens {
            let expectation = Expectation {
                constants: Vec::new(),
                instructions: vec![emitted],
            };
            state_expectation_test(vec![token], expectation);
        }
    }

    #[test]
    fn emit_grouping_chunk() {
        // 3 * (5 + 7)
        let input = vec![
            Token::number("3"),
            Token::multiply(),
            Token::with_type(TokenType::LeftParenthesis),
            Token::number("5.0"),
            Token::plus(),
            Token::number("7"),
            Token::with_type(TokenType::RightParenthesis),
        ];

        let expectation = Expectation {
            constants: vec![
                DataType::number(3.0),
                DataType::number(5.0),
                DataType::number(7.0),
            ],
            instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Constant(2),
                Instruction::Add,
                Instruction::Multiply,
            ],
        };

        state_expectation_test(input, expectation);
    }

    #[test]
    fn emit_string_constant() {
        let input = vec![Token::make(TokenType::String, "\"Text\"")];
        let expectation = Expectation {
            constants: vec![DataType::str_text("Text")],
            instructions: vec![],
        };
        state_expectation_test(input, expectation);
    }

    fn state_expectation_test(input: Vec<Token>, expectation: Expectation) {
        let mock = ScannerMock::new(input);
        let mut parser = Parser::with(Box::new(mock));
        let res = parser.compile();
        if let Err(err) = res {
            panic!("{:?}", err);
        }
        // assert!(res.is_ok());

        let chunk = parser.chunk;
        for (i, x) in expectation.constants.iter().enumerate() {
            assert_eq!(chunk.read_const(i as u8), Some(x.clone()));
        }

        let mut offset = 0;
        for instr_exp in expectation.instructions {
            let instr = chunk
                .fetch(&mut offset)
                .expect("Failed to fetch instruction");
            assert_eq!(instr, instr_exp);
        }
    }

    struct Expectation {
        constants: Vec<DataType>,
        instructions: Vec<Instruction>,
    }

    struct ScannerMock {
        idx: usize,
        tokens: Vec<Token>,
    }

    impl ScannerMock {
        fn new(tokens: Vec<Token>) -> Self {
            Self { idx: 0, tokens }
        }
    }

    impl TokenSource for ScannerMock {
        fn scan_token(&mut self) -> Token {
            let token = self.tokens.get(self.idx);
            if token.is_some() {
                self.idx += 1;
            }
            token.cloned().unwrap_or(Token::eof())
        }
    }

    impl Token {
        fn eof() -> Self {
            Self::with_type(TokenType::Eof)
        }

        fn number(value: &str) -> Self {
            Self::make(TokenType::Number, value)
        }

        fn minus() -> Self {
            Self::make(TokenType::Minus, "-")
        }

        fn plus() -> Self {
            Self::make(TokenType::Plus, "+")
        }

        fn multiply() -> Self {
            Self::make(TokenType::Star, "*")
        }

        fn divide() -> Self {
            Self::make(TokenType::Slash, "/")
        }

        fn with_type(t_type: TokenType) -> Self {
            Self::make(t_type, "")
        }

        fn make(t_type: TokenType, text: &str) -> Self {
            let position = CodePosition {
                line: 0,
                absolute_index: 0,
            };
            Self {
                t_type,
                text: text.to_string(),
                position,
            }
        }
    }
}

#[cfg(test)]
mod test_precedence {
    use super::*;

    #[test]
    fn increase_less_equal() {
        use Precedence::*;
        let precedence = [
            None, Assignment, Or, And, Equality, Comparison, Term, Factor, Unary, Call, Primary,
        ];

        for (i, item) in precedence.iter().enumerate() {
            let next = item.increased();
            assert!(item.le(item));
            assert!(item.le(&next));
            let next_val = precedence.get(i + 1).unwrap_or(&Precedence::Primary);
            assert_eq!(next, *next_val);
        }
    }
}
