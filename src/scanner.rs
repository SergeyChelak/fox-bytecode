use crate::token::{Token, TokenType};

pub struct Scanner {
    line: usize,
    code: Vec<char>,
    code_start_idx: usize,
    code_current_idx: usize,
}

#[derive(Debug)]
pub struct ScannerError {
    pub message: String,
    line: usize,
    code_idx: usize,
}

pub type ScannerResult<T> = Result<T, ScannerError>;

impl Scanner {
    pub fn new(code: Vec<char>) -> Self {
        Self {
            line: 1,
            code,
            code_start_idx: 0,
            code_current_idx: 0,
        }
    }

    pub fn scan_token(&mut self) -> ScannerResult<Token> {
        self.code_start_idx = self.code_current_idx;

        let Some(ch) = self.advance_char() else {
            return Ok(self.make_token(TokenType::Eof));
        };

        use TokenType::*;
        let token = match ch {
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
            _ => return Err(self.make_error("Unexpected character")),
        };
        Ok(token)
    }

    fn is_at_end(&self) -> bool {
        self.code_start_idx == self.code.len()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.code.get(self.code_current_idx);
        if ch.is_some() {
            self.code_current_idx += 1;
        }
        ch.cloned()
    }

    fn make_token(&self, t_type: TokenType) -> Token {
        Token {
            t_type,
            code_idx: self.code_start_idx,
            length: self.code_current_idx - self.code_start_idx,
            line: self.line,
        }
    }

    fn make_error(&self, message: &str) -> ScannerError {
        ScannerError {
            message: message.to_string(),
            line: self.line,
            code_idx: self.code_start_idx,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
            let mut scanner = Scanner::new(inp);
            let result = scanner.scan_token();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().t_type, t_type);
        }
    }
}
