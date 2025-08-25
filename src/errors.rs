use std::rc::Rc;

use crate::compiler::Token;

pub struct ErrorCollector {
    errors: Vec<ErrorInfo>,
    current: Token,
    previous: Token,
    panic_mode: bool,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Default::default(),
            current: Token::undefined(),
            previous: Token::undefined(),
            panic_mode: false,
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &Vec<ErrorInfo> {
        &self.errors
    }

    pub fn update_previous_token(&mut self, token: Token) {
        self.previous = token;
    }

    pub fn update_current_token(&mut self, token: Token) {
        self.current = token;
    }

    pub fn reset_panic(&mut self) {
        self.panic_mode = false;
    }

    pub fn is_panic(&self) -> bool {
        self.panic_mode
    }

    pub fn error_at_current(&mut self, message: &str) {
        self.push_error_info(self.current.clone(), message);
    }

    pub fn error(&mut self, message: &str) {
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
}

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    position: Option<CodePosition>,
    message: String,
}

impl ErrorInfo {
    pub fn with(token: Token, message: &str) -> Self {
        let append = |arr: &mut Vec<String>, value: String| {
            if value.is_empty() {
                return;
            }
            if arr.contains(&value) {
                return;
            }
            arr.push(value);
        };
        let mut combined = Vec::new();
        if token.is_err() {
            append(&mut combined, token.text);
        }
        append(&mut combined, message.to_string());
        let text = combined.join(" : ");
        Self {
            position: Some(token.position),
            message: text,
        }
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct CodePosition {
    pub line: usize,
    pub absolute_index: usize,
}

pub struct ErrorFormatter {
    code: Rc<Vec<char>>,
}

impl ErrorFormatter {
    pub fn with(code: Rc<Vec<char>>) -> Self {
        Self { code }
    }

    pub fn format_error(&self, info: &ErrorInfo) -> String {
        let Some(p) = &info.position else {
            return info.message.clone();
        };

        let mut lines: Vec<String> = Vec::new();
        let (offset, code_line) = self.extract_line(p);
        let prefix = format!("{} |", p.line);
        lines.push(format!("{}{}", prefix, code_line));

        let arrow_idx = prefix.len() + offset;
        let fill = " ".repeat(arrow_idx);
        lines.push(format!("{fill}▲"));

        let message = &info.message;
        if !message.is_empty() {
            let line = format!("{fill}└─ {message}",);
            lines.push(line)
        }

        lines.join("\n")
    }

    fn extract_line(&self, position: &CodePosition) -> (usize, String) {
        let mut left = position.absolute_index;
        let mut right = left;

        let is_terminator = |ch: char| -> bool { ch == '\n' || ch == '\r' };

        let len = self.code.len();
        let mut is_moving = true;
        while is_moving {
            is_moving = false;
            if left > 0 && !is_terminator(self.code[left - 1]) {
                is_moving = true;
                left -= 1;
            }

            if right < len - 1 && !is_terminator(self.code[right + 1]) {
                is_moving = true;
                right += 1;
            }
        }

        (
            position.absolute_index - left,
            self.code[left..=right].iter().collect::<String>(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl ErrorInfo {
        fn with_message(m: &str) -> Self {
            Self {
                position: None,
                message: m.to_string(),
            }
        }

        fn new(p: CodePosition, m: &str) -> Self {
            Self {
                position: Some(p),
                message: m.to_string(),
            }
        }
    }

    fn formatter_with_code(source: &str) -> ErrorFormatter {
        let code: Vec<char> = source.chars().collect();
        ErrorFormatter {
            code: Rc::new(code),
        }
    }

    #[test]
    fn format_error_empty_position() {
        let formatter = formatter_with_code("Line with some text");
        let info = ErrorInfo::with_message("Message");
        let output = formatter.format_error(&info);
        assert_eq!(output, "Message")
    }

    #[test]
    fn format_error_first_line() {
        let formatter = formatter_with_code("Line with some text\n2nd line");
        let pos = CodePosition {
            line: 1,
            absolute_index: 1,
        };
        let info = ErrorInfo::new(pos, "Message");
        let output = formatter.format_error(&info);
        assert!(output.starts_with("1 |Line with some text"));
        assert!(!output.contains("2nd line"));
        assert!(output.ends_with("Message"))
    }
}
