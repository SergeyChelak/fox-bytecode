use std::fmt::Display;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OpCode {
    Constant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,
}

pub enum Instruction {
    Constant(u8),
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

pub type FetchResult<T> = Result<T, FetchError>;

impl Instruction {
    // TODO: refactor to return result <Ok, End | Broken | Unknown>
    pub fn fetch(buffer: &[u8], offset: &mut usize) -> FetchResult<Self> {
        let byte = consume(buffer, offset).ok_or(FetchError::End)?;
        match byte {
            x if x == OpCode::Constant as u8 => {
                let arg1 = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::Constant(arg1))
            }
            x if x == OpCode::Negate as u8 => Ok(Instruction::Negate),

            x if x == OpCode::Add as u8 => Ok(Instruction::Add),
            x if x == OpCode::Subtract as u8 => Ok(Instruction::Subtract),
            x if x == OpCode::Multiply as u8 => Ok(Instruction::Multiply),
            x if x == OpCode::Divide as u8 => Ok(Instruction::Divide),

            x if x == OpCode::Return as u8 => Ok(Instruction::Return),
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
        let buffer = [OpCode::Return as u8];
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
        let buffer = [OpCode::Constant as u8, idx];
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
