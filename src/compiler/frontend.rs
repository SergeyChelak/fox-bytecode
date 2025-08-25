use crate::{
    ErrorInfo,
    compiler::{Token, TokenType, scanner::TokenSource},
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
        todo!()
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
        todo!()
    }

    // variables
    fn var_declaration(&mut self) {
        // let global = self.parse_variable("Expect variable name");

        // if self.is_match(TokenType::Equal) {
        //     self.expression();
        // } else {
        //     self.emit_instruction(&Instruction::Nil);
        // }
        // self.consume(
        //     TokenType::Semicolon,
        //     "Expect ';' after variable declaration",
        // );

        // self.define_variable(global);
    }

    // statements
    fn statement(&mut self) {
        todo!()
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
}

struct Compiler {
    //
}

impl Compiler {
    fn new() -> Self {
        Self {}
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
