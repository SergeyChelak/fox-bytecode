use crate::{
    ErrorInfo, Instruction, Value,
    compiler::{Local, LocalVariableInfo, MAX_SCOPE_SIZE, Token, TokenType, scanner::TokenSource},
};

pub struct Frontend {
    parser: Parser,
    compiler: Compiler,
    scanner: Box<dyn TokenSource>,
    panic_mode: bool,
    errors: Vec<ErrorInfo>,
}

impl Frontend {
    pub fn compile(&mut self) {
        self.advance();
        while !self.is_match(TokenType::Eof) {
            self.declaration();
        }
        self.end_compiler();
        todo!("check if error list empty, return valid value otherwise errors")
    }

    fn advance(&mut self) {
        self.parser.update_previous();
        let mut looping = true;
        while looping {
            let token = self.scanner.scan_token();
            let is_err = token.is_err();
            self.parser.set_current(token);
            if is_err {
                self.error_at_current("");
            }
            looping = is_err;
        }
    }

    fn is_match(&mut self, t_type: TokenType) -> bool {
        if !self.check(t_type) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&self, t_type: TokenType) -> bool {
        self.parser.cur_token_type() == t_type
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

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while !matches!(self.parser.cur_token_type(), TokenType::Eof) {
            if matches!(self.parser.prev_token_type(), TokenType::Semicolon) {
                return;
            }
            match self.parser.cur_token_type() {
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

    fn consume<T: AsRef<str>>(&mut self, t_type: TokenType, message: T) {
        if self.parser.cur_token_type() == t_type {
            self.advance();
            return;
        };

        self.error_at_current(message.as_ref());
    }

    fn end_compiler(&mut self) {
        self.emit_return();
    }
}

/// expressions
impl Frontend {
    fn expression(&mut self) {
        todo!()
    }
}

/// Variables
impl Frontend {
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
        if self.compiler.is_local_scope() {
            return 0;
        }
        self.identifier_constant(self.parser.previous.clone())
    }

    fn declare_variable(&mut self) {
        if self.compiler.is_global_scope() {
            return;
        }
        let token = self.parser.previous.clone();
        if self.compiler.has_declared_variable(&token) {
            self.error("Already a variable with this name in this scope");
        }
        self.add_local(token);
    }

    fn add_local(&mut self, token: Token) {
        if !self.compiler.has_capacity() {
            self.error("Too many local variables in function");
            return;
        }
        let local = Local::with_name(token.text);
        self.compiler.push_local(local);
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler.is_local_scope() {
            self.compiler.mark_initialized();
            return;
        }
        self.emit_instruction(&Instruction::DefineGlobal(global));
    }

    fn identifier_constant(&mut self, token: Token) -> u8 {
        self.make_constant(Value::text_from_string(token.text))
    }
}

/// Statements
impl Frontend {
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
        if self.is_match(TokenType::Switch) {
            self.switch_statement();
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

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_instruction(&Instruction::Pop);
    }

    fn if_statement(&mut self) {
        todo!()
    }

    /// Function implemented according to the book's grammar:
    /// switchStmt     → "switch" "(" expression ")"
    ///                  "{" switchCase* defaultCase? "}" ;
    /// switchCase     → "case" expression ":" statement* ;
    /// defaultCase    → "default" ":" statement* ;
    ///
    /// this isn't mainstream approach to use expressions in switch cases
    /// because their values are not known at compile time.
    /// As result, it may lead to unexpected behavior when
    /// different case entries will be associated with the same value
    fn switch_statement(&mut self) {
        todo!()
    }

    fn while_statement(&mut self) {
        todo!()
    }

    fn for_statement(&mut self) {
        todo!()
    }

    fn break_statement(&mut self) {
        todo!()
    }

    fn continue_statement(&mut self) {
        todo!()
    }

    fn begin_scope(&mut self) {
        self.compiler.begin_scope();
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block");
    }

    fn end_scope(&mut self) {
        self.compiler.end_scope(self.parser.get_line());
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_instruction(&Instruction::Print);
    }
}

/// Emit functions
impl Frontend {
    fn make_constant(&mut self, value: Value) -> u8 {
        todo!()
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> usize {
        let line = self.parser.get_line();
        self.compiler.emit_instruction_at_line(instruction, line)
    }

    fn emit_return(&mut self) -> usize {
        self.emit_instruction(&Instruction::Return)
    }
}

// Errors
impl Frontend {
    fn error_at_current(&mut self, message: &str) {
        self.push_error_info(self.parser.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.push_error_info(self.parser.previous.clone(), message);
    }

    // convenience function
    fn push_error_info(&mut self, elem: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        let info = ErrorInfo::with(elem, message);
        self.errors.push(info);
    }
}

struct Parser {
    current: Token,
    previous: Token,
}

impl Parser {
    fn new() -> Self {
        Self {
            current: Token::undefined(),
            previous: Token::undefined(),
        }
    }

    fn set_current(&mut self, token: Token) {
        self.current = token;
    }

    fn update_previous(&mut self) {
        self.previous = self.current.clone();
    }

    fn prev_token_type(&self) -> TokenType {
        self.previous.t_type
    }

    fn cur_token_type(&self) -> TokenType {
        self.current.t_type
    }

    fn get_line(&self) -> usize {
        self.previous.position.line
    }
}

struct Compiler {
    depth: usize,
    locals: Vec<Local>,
}

impl Compiler {
    fn new() -> Self {
        Self {
            depth: 0,
            locals: Default::default(),
        }
    }

    pub fn emit_instruction_at_line(&mut self, instruction: &Instruction, line: usize) -> usize {
        todo!()
    }

    // scope management
    pub fn begin_scope(&mut self) {
        todo!()
    }

    pub fn end_scope(&mut self, line: usize) {
        todo!()
    }

    pub fn is_local_scope(&self) -> bool {
        self.depth > 0
    }

    pub fn is_global_scope(&self) -> bool {
        self.depth == 0
    }

    pub fn has_declared_variable(&self, token: &Token) -> bool {
        todo!()
    }

    pub fn has_capacity(&self) -> bool {
        self.locals.len() < MAX_SCOPE_SIZE
    }

    pub fn push_local(&mut self, local: Local) {
        self.locals.push(local);
    }

    pub fn mark_initialized(&mut self) {
        let Some(local) = self.locals.last_mut() else {
            panic!();
        };
        local.depth = Some(self.depth);
    }

    pub fn resolve_local(&self, token: &Token) -> Option<LocalVariableInfo> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::compiler::scanner::tests::ScannerMock;

    use super::*;

    #[test]
    fn advance_test_normal() {
        let mut frontend = compose_frontend_with_tokens(vec![Token::minus(), Token::number("123")]);
        frontend.advance();
        assert_eq!(frontend.parser.previous, Token::undefined());
        assert_eq!(frontend.parser.current, Token::minus());
    }

    #[test]
    fn advance_test_error() {
        let mut frontend =
            compose_frontend_with_tokens(vec![Token::error("wrong"), Token::number("123")]);
        assert!(!frontend.panic_mode);
        frontend.advance();
        assert_eq!(frontend.parser.previous, Token::undefined());
        assert_eq!(frontend.parser.current, Token::number("123"));
        assert!(frontend.panic_mode);
    }

    #[test]
    fn match_test_true() {
        let mut frontend =
            compose_frontend_with_tokens(vec![Token::plus(), Token::minus(), Token::number("123")]);
        // call advance to fill initial undefined token's value
        frontend.advance();
        assert!(frontend.is_match(TokenType::Plus));
        assert_eq!(frontend.parser.previous, Token::plus());
        assert_eq!(frontend.parser.current, Token::minus());
    }

    #[test]
    fn match_test_false() {
        let mut frontend =
            compose_frontend_with_tokens(vec![Token::plus(), Token::minus(), Token::number("123")]);
        // call advance to fill initial undefined token's value
        frontend.advance();
        assert!(!frontend.is_match(TokenType::False));
        assert_eq!(frontend.parser.previous, Token::undefined());
        assert_eq!(frontend.parser.current, Token::plus());
    }

    //
    fn compose_frontend_with_tokens(tokens: Vec<Token>) -> Frontend {
        let scanner = ScannerMock::new(tokens);
        compose_frontend(Box::new(scanner))
    }

    fn compose_frontend(scanner: Box<dyn TokenSource>) -> Frontend {
        Frontend {
            parser: Parser::new(),
            compiler: Compiler::new(),
            scanner,
            panic_mode: false,
            errors: Vec::new(),
        }
    }
}
