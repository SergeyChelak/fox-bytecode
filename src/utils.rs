use std::rc::Rc;

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

    pub fn format_error<T: AsRef<str>>(
        &self,
        position: Option<CodePosition>,
        message: T,
    ) -> String {
        let Some(p) = &position else {
            return message.as_ref().to_string();
        };

        let mut lines: Vec<String> = Vec::new();
        let (offset, code_line) = self.extract_line(p);
        let prefix = format!("{} |", p.line);
        lines.push(format!("{}{}", prefix, code_line));

        let arrow_idx = prefix.len() + offset;
        let fill = " ".repeat(arrow_idx);
        lines.push(format!("{fill}▲"));

        let message = message.as_ref();
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

    fn formatter_with_code(source: &str) -> ErrorFormatter {
        let code: Vec<char> = source.chars().collect();
        ErrorFormatter {
            code: Rc::new(code),
        }
    }

    #[test]
    fn format_error_empty_position() {
        let formatter = formatter_with_code("Line with some text");
        let output = formatter.format_error(None, "Message");
        assert_eq!(output, "Message")
    }

    #[test]
    fn format_error_first_line() {
        let formatter = formatter_with_code("Line with some text\n2nd line");
        let pos = CodePosition {
            line: 1,
            absolute_index: 1,
        };
        let output = formatter.format_error(Some(pos), "Message");
        assert!(output.starts_with("1 |Line with some text"));
        assert!(!output.contains("2nd line"));
        assert!(output.ends_with("Message"))
    }
}
