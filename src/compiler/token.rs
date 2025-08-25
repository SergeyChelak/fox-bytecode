use crate::errors::CodePosition;

#[derive(Debug, Clone)]
pub struct Token {
    pub t_type: TokenType,
    pub text: String,
    pub position: CodePosition,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Undefined,
    // single-character tokens
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Case,
    Colon,
    Comma,
    DefaultCase,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // 1 or 2 chars tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // literals
    Identifier,
    String,
    Number,
    // Keywords
    And,
    Break,
    Class,
    Continue,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    Switch,
    This,
    True,
    Var,
    While,
    //
    Error,
    Eof,
}

impl Token {
    pub fn is_err(&self) -> bool {
        matches!(self.t_type, TokenType::Error)
    }

    pub fn undefined() -> Self {
        Self::with_type(TokenType::Undefined)
    }

    pub fn with_type(t_type: TokenType) -> Self {
        Self::make(t_type, "")
    }

    pub fn make(t_type: TokenType, text: &str) -> Self {
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
