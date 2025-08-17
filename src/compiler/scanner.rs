use std::rc::Rc;

use crate::utils::CodePosition;

use super::{Token, TokenType};

pub trait TokenSource {
    fn scan_token(&mut self) -> Token;
}

pub struct Scanner {
    line: usize,
    code: Rc<Vec<char>>,
    code_start_idx: usize,
    code_current_idx: usize,
}

impl TokenSource for Scanner {
    fn scan_token(&mut self) -> Token {
        self.fetch_next_token()
    }
}

impl Scanner {
    pub fn new(code: Rc<Vec<char>>) -> Self {
        Self {
            line: 1,
            code,
            code_start_idx: 0,
            code_current_idx: 0,
        }
    }

    fn fetch_next_token(&mut self) -> Token {
        self.skip_non_code();
        self.code_start_idx = self.code_current_idx;

        let Some(ch) = self.advance_char() else {
            return self.make_token(TokenType::Eof);
        };

        use TokenType::*;
        match ch {
            '(' => self.make_token(LeftParenthesis),
            ')' => self.make_token(RightParenthesis),
            '{' => self.make_token(LeftBrace),
            '}' => self.make_token(RightBrace),
            ';' => self.make_token(Semicolon),
            ',' => self.make_token(Comma),
            '.' => self.make_token(Dot),
            '-' => self.make_token(Minus),
            '+' => self.make_token(Plus),
            '/' => self.make_token(Slash),
            '*' => self.make_token(Star),
            '!' => {
                let cond = self.match_char('=');
                self.condition_make_token(cond, BangEqual, Bang)
            }
            '=' => {
                let cond = self.match_char('=');
                self.condition_make_token(cond, EqualEqual, Equal)
            }
            '<' => {
                let cond = self.match_char('=');
                self.condition_make_token(cond, LessEqual, Less)
            }
            '>' => {
                let cond = self.match_char('=');
                self.condition_make_token(cond, GreaterEqual, Greater)
            }
            '"' => self.advance_string_token(),
            x if is_alphabetic(x) => self.advance_identifier(),
            x if x.is_ascii_digit() => self.advance_number_token(),
            x => self.make_error_token(&format!("Unexpected character '{x}'")),
        }
    }

    fn skip_non_code(&mut self) {
        while self.skip_whitespace() || self.skip_comment_line() {}
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut skipped = false;
        while let Some(ch) = self.peek_char() {
            if !ch.is_ascii_whitespace() {
                break;
            }
            self.advance_char();
            if ch == '\n' {
                self.line += 1;
            }
            skipped = true;
        }
        skipped
    }

    fn skip_comment_line(&mut self) -> bool {
        if (Some('/'), Some('/')) != (self.peek_char(), self.peek_next_char()) {
            return false;
        }
        while let Some(ch) = self.peek_char() {
            self.advance_char();
            if ch == '\n' {
                break;
            }
        }
        true
    }

    fn peek_char(&self) -> Option<char> {
        self.code.get(self.code_current_idx).cloned()
    }

    fn peek_next_char(&self) -> Option<char> {
        self.code.get(self.code_current_idx + 1).cloned()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char();
        if ch.is_some() {
            self.code_current_idx += 1;
        }
        ch
    }

    fn advance_identifier(&mut self) -> Token {
        while let Some(ch) = self.peek_char() {
            if !is_alphanumeric(ch) {
                break;
            }
            self.advance_char();
        }

        let value = self.current_lexeme();

        use TokenType::*;
        let t_type = match value.as_str() {
            "and" => And,
            "class" => Class,
            "else" => Else,
            "false" => False,
            "for" => For,
            "fun" => Fun,
            "if" => If,
            "nil" => Nil,
            "or" => Or,
            "print" => Print,
            "return" => Return,
            "super" => Super,
            "this" => This,
            "true" => True,
            "var" => Var,
            "while" => While,
            _ => Identifier,
        };
        self.make_token(t_type)
    }

    fn current_lexeme(&self) -> String {
        self.code[self.code_start_idx..self.code_current_idx]
            .iter()
            .collect::<std::string::String>()
    }

    fn advance_string_token(&mut self) -> Token {
        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => break,
                '\n' => self.line += 1,
                _ => {}
            }
            self.advance_char();
        }
        if self.peek_char().is_none() {
            return self.make_error_token("Unterminated string");
        }
        self.advance_char();
        self.make_token(TokenType::String)
    }

    fn advance_number_token(&mut self) -> Token {
        while let Some(ch) = self.peek_char() {
            if !ch.is_ascii_digit() {
                break;
            }
            self.advance_char();
        }

        if Some('.') == self.peek_char()
            && self
                .peek_next_char()
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
        {
            self.advance_char();
        }

        while let Some(ch) = self.peek_char() {
            if !ch.is_ascii_digit() {
                break;
            }
            self.advance_char();
        }

        self.make_token(TokenType::Number)
    }

    fn match_char(&mut self, expected: char) -> bool {
        let Some(ch) = self.peek_char() else {
            return false;
        };
        if ch != expected {
            return false;
        }
        self.code_current_idx += 1;
        true
    }

    fn condition_make_token(
        &self,
        condition: bool,
        true_case: TokenType,
        false_case: TokenType,
    ) -> Token {
        let t_type = if condition { true_case } else { false_case };
        self.make_token(t_type)
    }

    fn make_token(&self, t_type: TokenType) -> Token {
        Token {
            t_type,
            text: self.current_lexeme(),
            position: self.code_position(),
        }
    }

    fn make_error_token(&self, message: &str) -> Token {
        Token {
            t_type: TokenType::Error,
            text: message.to_string(),
            position: self.code_position(),
        }
    }

    fn code_position(&self) -> CodePosition {
        CodePosition {
            line: self.line,
            absolute_index: self.code_start_idx,
        }
    }
}

fn is_alphabetic(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_alphanumeric(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod test {
    use super::*;

    impl Scanner {
        fn with_raw_code(code: Vec<char>) -> Self {
            Self::new(Rc::new(code))
        }
    }

    #[test]
    fn scan_single_char_token() {
        let data = [
            ('(', TokenType::LeftParenthesis),
            (')', TokenType::RightParenthesis),
            ('{', TokenType::LeftBrace),
            ('}', TokenType::RightBrace),
            (';', TokenType::Semicolon),
            (',', TokenType::Comma),
            ('.', TokenType::Dot),
            ('-', TokenType::Minus),
            ('+', TokenType::Plus),
            ('/', TokenType::Slash),
            ('*', TokenType::Star),
        ];

        for (ch, t_type) in data {
            let inp = vec![ch];
            let mut scanner = Scanner::with_raw_code(inp);
            let result = scanner.scan_token();
            assert_eq!(result.t_type, t_type);

            let result = scanner.scan_token();
            assert!(matches!(result.t_type, TokenType::Eof));
        }
    }

    #[test]
    fn scan_double_char_token() {
        use TokenType::*;
        let cases = [
            ("!", Bang),
            ("!=", BangEqual),
            ("=", Equal),
            ("==", EqualEqual),
            ("<", Less),
            ("<=", LessEqual),
            (">", Greater),
            (">=", GreaterEqual),
        ];
        for (ch, t_type) in cases {
            let inp = ch.chars().collect::<Vec<_>>();

            let mut scanner = Scanner::with_raw_code(inp);
            let result = scanner.scan_token();
            assert_eq!(result.t_type, t_type);

            let result = scanner.scan_token();
            assert!(matches!(result.t_type, TokenType::Eof));
        }
    }

    #[test]
    fn scan_skip_whitespace() {
        let code = "\t. ,\n!".chars().collect::<Vec<_>>();

        let mut scanner = Scanner::with_raw_code(code);
        assert_eq!(scanner.scan_token().t_type, TokenType::Dot);
        assert_eq!(scanner.scan_token().t_type, TokenType::Comma);
        assert_eq!(scanner.scan_token().t_type, TokenType::Bang);
    }

    #[test]
    fn scan_skip_comment_line() {
        let code = "\n    // hello\n . // world\n   \t,"
            .chars()
            .collect::<Vec<_>>();
        let mut scanner = Scanner::with_raw_code(code);
        assert_eq!(scanner.scan_token().t_type, TokenType::Dot);
        assert_eq!(scanner.scan_token().t_type, TokenType::Comma);
    }

    #[test]
    fn scan_literal_token() {
        let code = "\"1234\"".chars().collect::<Vec<_>>();
        let mut scanner = Scanner::with_raw_code(code);
        let token = scanner.scan_token();
        assert_eq!(token.t_type, TokenType::String);
        assert_eq!(token.text, "\"1234\"");
    }

    #[test]
    fn scan_literal_token_unterminated() {
        let code = "\"1234".chars().collect::<Vec<_>>();
        let mut scanner = Scanner::with_raw_code(code);
        assert!(scanner.scan_token().is_err());
    }

    #[test]
    fn scan_numbers() {
        let cases = ["1", "123", "12.23"];
        for case in cases {
            let code = case.chars().collect::<Vec<_>>();
            let mut scanner = Scanner::with_raw_code(code);
            let token = scanner.scan_token();
            assert_eq!(token.t_type, TokenType::Number);
        }
    }

    #[test]
    fn scan_keywords_identifiers() {
        use TokenType::*;
        let cases = [
            ("and", And),
            ("class", Class),
            ("else", Else),
            ("false", False),
            ("for", For),
            ("fun", Fun),
            ("if", If),
            ("nil", Nil),
            ("or", Or),
            ("print", Print),
            ("return", Return),
            ("super", Super),
            ("this", This),
            ("true", True),
            ("var", Var),
            ("while", While),
            ("my_identifier", Identifier),
            ("__myId2", Identifier),
        ];
        for (case, t_type) in cases {
            let code = case.chars().collect::<Vec<_>>();
            let mut scanner = Scanner::with_raw_code(code);
            let token = scanner.scan_token();
            assert_eq!(token.t_type, t_type);
        }
    }
}
