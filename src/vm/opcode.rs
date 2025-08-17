use std::fmt::Display;

pub const OPCODE_CONSTANT: u8 = 0;
pub const OPCODE_NIL: u8 = 1;
pub const OPCODE_TRUE: u8 = 2;
pub const OPCODE_FALSE: u8 = 3;
pub const OPCODE_NEGATE: u8 = 4;
pub const OPCODE_ADD: u8 = 5;
pub const OPCODE_SUBTRACT: u8 = 6;
pub const OPCODE_MULTIPLY: u8 = 7;
pub const OPCODE_DIVIDE: u8 = 8;
pub const OPCODE_RETURN: u8 = 9;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Constant(u8),
    Nil,
    True,
    False,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Constant(x) => write!(f, "const {x}"),
            Instruction::Nil => write!(f, "nil"),
            Instruction::True => write!(f, "true"),
            Instruction::False => write!(f, "false"),
            Instruction::Negate => write!(f, "negate"),
            Instruction::Add => write!(f, "add"),
            Instruction::Subtract => write!(f, "subtract"),
            Instruction::Multiply => write!(f, "multiply"),
            Instruction::Divide => write!(f, "divide"),
            Instruction::Return => write!(f, "return"),
        }
    }
}

#[derive(Debug)]
pub enum FetchError {
    Unknown(u8),
    Broken,
    End,
}

impl Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Unknown(x) => write!(f, "Unknown instruction {x}"),
            FetchError::Broken => write!(f, "Broken instruction"),
            FetchError::End => write!(f, "End of program"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<u8>> for Instruction {
    fn into(self) -> Vec<u8> {
        match self {
            Instruction::Constant(val) => vec![OPCODE_CONSTANT, val],
            Instruction::Nil => vec![OPCODE_NIL],
            Instruction::True => vec![OPCODE_TRUE],
            Instruction::False => vec![OPCODE_FALSE],
            Instruction::Negate => vec![OPCODE_NEGATE],
            Instruction::Add => vec![OPCODE_ADD],
            Instruction::Subtract => vec![OPCODE_SUBTRACT],
            Instruction::Multiply => vec![OPCODE_MULTIPLY],
            Instruction::Divide => vec![OPCODE_DIVIDE],
            Instruction::Return => vec![OPCODE_RETURN],
        }
    }
}

pub type FetchResult<T> = Result<T, FetchError>;

impl Instruction {
    pub fn fetch(buffer: &[u8], offset: &mut usize) -> FetchResult<Self> {
        let byte = consume(buffer, offset).ok_or(FetchError::End)?;
        match byte {
            OPCODE_CONSTANT => {
                let arg1 = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::Constant(arg1))
            }
            OPCODE_NEGATE => Ok(Instruction::Negate),

            OPCODE_NIL => Ok(Instruction::Nil),
            OPCODE_TRUE => Ok(Instruction::True),
            OPCODE_FALSE => Ok(Instruction::False),

            OPCODE_ADD => Ok(Instruction::Add),
            OPCODE_SUBTRACT => Ok(Instruction::Subtract),
            OPCODE_MULTIPLY => Ok(Instruction::Multiply),
            OPCODE_DIVIDE => Ok(Instruction::Divide),

            OPCODE_RETURN => Ok(Instruction::Return),
            x => Err(FetchError::Unknown(x)),
        }
    }
}

fn consume(buffer: &[u8], offset: &mut usize) -> Option<u8> {
    let byte = buffer.get(*offset)?;
    *offset += 1;
    Some(*byte)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn instruction_fetch_none() {
        let buffer: [u8; 0] = [];
        let mut offset = 0;
        let instr = Instruction::fetch(&buffer, &mut offset);
        assert!(instr.is_err());
    }

    #[test]
    fn instruction_fetch_return() {
        let buffer = [OPCODE_RETURN];
        let mut offset = 0;
        let instr = Instruction::fetch(&buffer, &mut offset);
        assert!(instr.is_ok());
        assert_eq!(offset, 1);
        let instr = instr.unwrap();
        assert!(matches!(instr, Instruction::Return))
    }

    #[test]
    fn instruction_fetch_constant() {
        let idx = 3u8;
        let buffer = [OPCODE_CONSTANT, idx];
        let mut offset = 0;
        let instr = Instruction::fetch(&buffer, &mut offset);
        assert!(instr.is_ok());
        assert_eq!(offset, 2);
        match instr.unwrap() {
            Instruction::Constant(x) => assert_eq!(x, idx),
            _ => panic!("Invalid opcode"),
        }
    }
}
