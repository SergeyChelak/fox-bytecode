use crate::{
    chunk::Chunk,
    compiler::{
        scanner::TokenSource,
        scope::{Local, Scope},
        token::{Token, TokenType},
    },
    data::DataType,
    error_info::ErrorInfo,
    utils::jump_to_bytes,
    vm::Instruction,
};

pub struct Parser {
    current: Token,
    previous: Token,
    scanner: Box<dyn TokenSource>,
    panic_mode: bool,
    chunk: Chunk,
    errors: Vec<ErrorInfo>,
    scope: Scope,
    loop_stack: Vec<LoopData>,
}

impl Parser {
    pub fn with(scanner: Box<dyn TokenSource>) -> Self {
        Self {
            current: Token::undefined(),
            previous: Token::undefined(),
            scanner,
            panic_mode: false,
            chunk: Chunk::new(),
            errors: Default::default(),
            scope: Default::default(),
            loop_stack: Default::default(),
        }
    }

    pub fn compile(mut self) -> Result<Chunk, Vec<ErrorInfo>> {
        self.advance();
        while !self.is_match(TokenType::Eof) {
            self.declaration();
        }
        self.end_compiler();
        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        Ok(self.chunk)
    }

    fn declaration(&mut self) {
        if self.is_match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name");

        if self.is_match(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_instruction(&Instruction::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        );

        self.define_variable(global);
    }

    fn parse_variable(&mut self, message: &str) -> u8 {
        self.consume(TokenType::Identifier, message);
        self.declare_variable();
        if self.scope.is_local() {
            return 0;
        }
        self.identifier_constant(self.previous.clone())
    }

    fn declare_variable(&mut self) {
        if self.scope.is_global() {
            return;
        }
        let token = self.previous.clone();
        if self.scope.has_declared_variable(&token) {
            self.error("Already a variable with this name in this scope");
        }
        self.add_local(token);
    }

    fn add_local(&mut self, token: Token) {
        if !self.scope.has_capacity() {
            self.error("Too many local variables in function");
            return;
        }
        let local = Local::with_token(token);
        self.scope.push(local);
    }

    fn identifier_constant(&mut self, token: Token) -> u8 {
        self.make_constant(DataType::text_from_string(token.text))
    }

    fn define_variable(&mut self, global: u8) {
        if self.scope.is_local() {
            self.scope.mark_initialized();
            return;
        }
        self.emit_instruction(&Instruction::DefineGlobal(global));
    }

    fn statement(&mut self) {
        if self.is_match(TokenType::Print) {
            self.print_statement();
            return;
        }
        if self.is_match(TokenType::Break) {
            self.break_statement();
            return;
        }
        if self.is_match(TokenType::Continue) {
            self.continue_statement();
            return;
        }
        if self.is_match(TokenType::For) {
            self.for_statement();
            return;
        }
        if self.is_match(TokenType::If) {
            self.if_statement();
            return;
        }
        if self.is_match(TokenType::While) {
            self.while_statement();
            return;
        }
        if self.is_match(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
            return;
        }
        self.expression_statement();
    }

    fn begin_scope(&mut self) {
        self.scope.begin_scope();
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block");
    }

    fn end_scope(&mut self) {
        let pops = self.scope.end_scope();
        for _ in 0..pops {
            self.emit_instruction(&Instruction::Pop);
        }
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'for'");
        if self.is_match(TokenType::Semicolon) {
            // no initializer
        } else if self.is_match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.mark_start_loop();
        let mut exit_jump: Option<usize> = None;
        if !self.is_match(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition");
            exit_jump = Some(self.emit_instruction(&Instruction::stub_jump_if_false()));
            self.emit_instruction(&Instruction::Pop);
        }

        if !self.is_match(TokenType::RightParenthesis) {
            let body_jump = self.emit_instruction(&Instruction::stub_jump());
            let increment_start = self.chunk.len();
            self.expression();
            self.emit_instruction(&Instruction::Pop);
            self.consume(TokenType::RightParenthesis, "Expect ')' after for clauses");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_instruction(&Instruction::Pop); // condition
        }

        self.flush_loop();
        self.end_scope();
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_instruction(&Instruction::Pop);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'if'");
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expect ')' after condition");

        let then_jump = self.emit_instruction(&Instruction::stub_jump_if_false());
        self.emit_instruction(&Instruction::Pop);
        self.statement();

        let else_jump = self.emit_instruction(&Instruction::stub_jump());

        self.patch_jump(then_jump);
        self.emit_instruction(&Instruction::Pop);

        if self.is_match(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let loop_start = self.mark_start_loop();
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'while'");
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expect ')' after condition");

        let exit_jump = self.emit_instruction(&Instruction::stub_jump_if_false());
        self.emit_instruction(&Instruction::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.flush_loop();
        self.patch_jump(exit_jump);
        self.emit_instruction(&Instruction::Pop);
    }

    fn mark_start_loop(&mut self) -> usize {
        let start = self.chunk.len();
        let data = LoopData::new(start);
        self.loop_stack.push(data);
        start
    }

    fn flush_loop(&mut self) {
        let Some(val) = self.loop_stack.pop() else {
            self.error("Bug: loop_stack is broken");
            return;
        };
        for exit_jump in val.breaks {
            self.patch_jump(exit_jump);
        }
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let instr = Instruction::Loop(0x0, 0x0);
        let size = instr.size();
        let offset = self.chunk.len() - loop_start + size;
        if offset > u16::MAX as usize {
            self.error("Loop body too large");
        }
        let (f, s) = jump_to_bytes(offset);
        self.emit_instruction(&Instruction::Loop(f, s));
    }

    fn continue_statement(&mut self) {
        self.consume(TokenType::Semicolon, "Expect ';' after 'continue'");
        let Some(data) = self.loop_stack.last() else {
            self.error("'continue' statement allowed inside loops only");
            return;
        };
        self.emit_loop(data.start);
    }

    fn break_statement(&mut self) {
        self.consume(TokenType::Semicolon, "Expect ';' after 'break'");
        if self.loop_stack.is_empty() {
            self.error("'break' statement allowed inside loops only");
        }
        let offset = self.emit_instruction(&Instruction::stub_jump());
        self.emit_instruction(&Instruction::Pop);
        let Some(data) = self.loop_stack.last_mut() else {
            self.error("Bug: loop stack became empty");
            return;
        };
        data.breaks.push(offset);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_instruction(&Instruction::Print);
    }

    fn is_match(&mut self, t_type: TokenType) -> bool {
        if !self.check(t_type) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&self, t_type: TokenType) -> bool {
        self.cur_token_type() == t_type
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            let token = self.scanner.scan_token();
            let is_err = token.is_err();
            self.current = token;
            if is_err {
                self.error_at_current("");
            } else {
                break;
            }
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while !matches!(self.cur_token_type(), TokenType::Eof) {
            if matches!(self.prev_token_type(), TokenType::Semicolon) {
                return;
            }
            match self.cur_token_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.push_error_info(self.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.push_error_info(self.previous.clone(), message);
    }

    fn push_error_info(&mut self, elem: Token, message: &str) {
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
        if self.current.t_type == t_type {
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

        let can_assign = precedence.le(&Precedence::Assignment);
        prefix_rule(self, can_assign);

        while precedence.le(&self.get_rule(self.cur_token_type()).precedence) {
            self.advance();
            let infix_rule = self
                .get_rule(self.prev_token_type())
                .infix
                .expect("Infix is none");
            infix_rule(self, can_assign);
        }

        if can_assign && self.is_match(TokenType::Equal) {
            self.error("Invalid assignment target");
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();
    }

    fn emit_return(&mut self) {
        self.emit_instruction(&Instruction::Return);
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_instruction(&Instruction::stub_jump_if_false());
        self.emit_instruction(&Instruction::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_instruction(&Instruction::stub_jump_if_false());
        let end_jump = self.emit_instruction(&Instruction::stub_jump());

        self.patch_jump(else_jump);
        self.emit_instruction(&Instruction::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn number(&mut self, _can_assign: bool) {
        // I don't like this approach
        // according to strtod it returns 0.0 as fallback
        let value = DataType::number_from(&self.previous.text).unwrap_or(DataType::Number(0.0));
        self.emit_constant(value)
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParenthesis, "Expect ')' after expression");
    }

    fn unary(&mut self, _can_assign: bool) {
        // TODO: made according to the book, looks bad...
        let operator_type = self.prev_token_type();
        // Compile the operand
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction
        match operator_type {
            TokenType::Minus => self.emit_instruction(&Instruction::Negate),
            TokenType::Bang => self.emit_instruction(&Instruction::Not),
            _ => unreachable!("unary"),
        };
    }

    fn binary(&mut self, _can_assign: bool) {
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

    fn literal(&mut self, _can_assign: bool) {
        match self.prev_token_type() {
            TokenType::False => self.emit_instruction(&Instruction::False),
            TokenType::True => self.emit_instruction(&Instruction::True),
            TokenType::Nil => self.emit_instruction(&Instruction::Nil),
            _ => unreachable!("literal"),
        };
    }

    fn string(&mut self, _can_assign: bool) {
        let s = self.previous.text.as_str();
        let text = &s[1..s.len() - 1];
        self.emit_constant(DataType::text_from_str(text));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.clone(), can_assign);
    }

    fn named_variable(&mut self, token: Token, can_assign: bool) {
        let (getter, setter) = if let Some(info) = self.scope.resolve_local(&token) {
            if info.depth.is_none() {
                self.error("Can't read local variable in its own initializer");
            }
            (
                Instruction::GetLocal(info.index),
                Instruction::SetLocal(info.index),
            )
        } else {
            let idx = self.identifier_constant(token);
            (Instruction::GetGlobal(idx), Instruction::SetGlobal(idx))
        };
        if can_assign && self.is_match(TokenType::Equal) {
            self.expression();
            self.emit_instruction(&setter);
        } else {
            self.emit_instruction(&getter);
        }
    }

    fn prev_token_type(&self) -> TokenType {
        self.previous.t_type
    }

    fn cur_token_type(&self) -> TokenType {
        self.current.t_type
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
            Identifier => ParseRule::new(Some(Self::variable), None, Precedence::None),
            And => ParseRule::new(None, Some(Self::and), Precedence::And),
            Or => ParseRule::new(None, Some(Self::or), Precedence::Or),
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

    fn emit_instruction(&mut self, instruction: &Instruction) -> usize {
        let start = self.chunk.len();
        let line = self.previous.position.line;
        let bytes: Vec<u8> = instruction.as_vec();
        for byte in bytes.into_iter() {
            self.chunk.write_u8(byte, line);
        }
        start
    }

    fn patch_jump(&mut self, offset: usize) {
        let (fetch_result, size) = {
            let mut idx = offset;
            let res = self.chunk.fetch(&mut idx);
            let size = idx - offset;
            (res, size)
        };

        let jump = self.chunk.len() - offset - size;
        if jump > u16::MAX as usize {
            self.error("Too much code to jump over");
        }
        let (first, second) = jump_to_bytes(jump);
        let instr = match fetch_result {
            Ok(Instruction::JumpIfFalse(_, _)) => Instruction::JumpIfFalse(first, second),
            Ok(Instruction::Jump(_, _)) => Instruction::Jump(first, second),
            Err(err) => {
                self.error(&format!("Bug: {err}"));
                return;
            }
            _ => {
                self.error("Bug: Attempt to patch non-jump instruction in 'path_jump' function");
                return;
            }
        };
        self.patch_instruction(&instr, offset);
    }

    fn patch_instruction(&mut self, instruction: &Instruction, offset: usize) {
        let bytes: Vec<u8> = instruction.as_vec();
        for (idx, byte) in bytes.into_iter().enumerate() {
            self.chunk.patch_u8(byte, offset + idx);
        }
    }

    fn emit_instructions(&mut self, instruction: &[Instruction]) {
        instruction
            .iter()
            .for_each(|inst| _ = self.emit_instruction(inst));
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

type ParseFn = fn(&mut Parser, bool);

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

struct LoopData {
    start: usize,
    breaks: Vec<usize>,
}

impl LoopData {
    fn new(start: usize) -> Self {
        Self {
            start,
            breaks: Default::default(),
        }
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;

    #[test]
    fn patch_instruction() {
        let mock = ScannerMock::new(vec![]);
        let mut parser = Parser::with(Box::new(mock));
        parser.emit_instruction(&Instruction::Add);
        let emit_addr = parser.emit_instruction(&Instruction::Constant(1));
        parser.emit_instruction(&Instruction::Subtract);
        parser.emit_instruction(&Instruction::Return);
        parser.patch_instruction(&Instruction::Constant(2), emit_addr);

        let chunk = parser.chunk;
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

    #[test]
    fn emit_unary_chunk() {
        let input = vec![Token::minus(), Token::number("12.345"), Token::semicolon()];
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
            let input = vec![
                Token::number("3"),
                token,
                Token::number("5.0"),
                Token::semicolon(),
            ];
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
            state_expectation_test(vec![token, Token::semicolon()], expectation);
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
            Token::semicolon(),
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
        let input = vec![
            Token::make(TokenType::String, "\"Text\""),
            Token::semicolon(),
        ];
        let expectation = Expectation {
            constants: vec![DataType::text_from_str("Text")],
            instructions: vec![],
        };
        state_expectation_test(input, expectation);
    }

    fn state_expectation_test(input: Vec<Token>, expectation: Expectation) {
        let mock = ScannerMock::new(input);
        let parser = Parser::with(Box::new(mock));

        let chunk = match parser.compile() {
            Ok(value) => value,
            Err(err) => panic!("{:?}", err),
        };
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

        fn semicolon() -> Self {
            Self::make(TokenType::Semicolon, ";")
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
