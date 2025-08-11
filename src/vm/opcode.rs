use std::fmt::Display;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OpCode {
    Constant,
    Return,
}

pub enum Instruction {
    Constant(u8),
    Return,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Constant(x) => write!(f, "const {x}"),
            Instruction::Return => write!(f, "return"),
        }
    }
}

impl Instruction {
    pub fn fetch(buffer: &[u8], offset: &mut usize) -> Option<Self> {
        let byte = consume(buffer, offset)?;
        match byte {
            x if x == OpCode::Constant as u8 => {
                let arg1 = consume(buffer, offset)?;
                Some(Instruction::Constant(arg1))
            }
            x if x == OpCode::Return as u8 => Some(Instruction::Return),
            _ => None,
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
