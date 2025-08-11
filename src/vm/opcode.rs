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

impl Instruction {
    // TODO: refactor to return result <Ok, End | Broken | Unknown>
    pub fn fetch(buffer: &[u8], offset: &mut usize) -> Option<Self> {
        let byte = consume(buffer, offset)?;
        match byte {
            x if x == OpCode::Constant as u8 => {
                let arg1 = consume(buffer, offset)?;
                Some(Instruction::Constant(arg1))
            }
            x if x == OpCode::Negate as u8 => Some(Instruction::Negate),

            x if x == OpCode::Add as u8 => Some(Instruction::Add),
            x if x == OpCode::Subtract as u8 => Some(Instruction::Subtract),
            x if x == OpCode::Multiply as u8 => Some(Instruction::Multiply),
            x if x == OpCode::Divide as u8 => Some(Instruction::Divide),

            x if x == OpCode::Return as u8 => Some(Instruction::Return),
            _ => panic!("Unexpected opcode {byte}"),
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
        assert!(instr.is_none());
    }

    #[test]
    fn instruction_fetch_return() {
        let buffer = [OpCode::Return as u8];
        let mut offset = 0;
        let instr = Instruction::fetch(&buffer, &mut offset);
        assert!(instr.is_some());
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
        assert!(instr.is_some());
        assert_eq!(offset, 2);
        match instr.unwrap() {
            Instruction::Constant(x) => assert_eq!(x, idx),
            _ => panic!("Invalid opcode"),
        }
    }
}
