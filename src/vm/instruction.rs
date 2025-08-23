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
pub const OPCODE_NOT: u8 = 10;
pub const OPCODE_LESS: u8 = 11;
pub const OPCODE_GREATER: u8 = 12;
pub const OPCODE_EQUAL: u8 = 13;
pub const OPCODE_PRINT: u8 = 14;
pub const OPCODE_POP: u8 = 15;
pub const OPCODE_DEFINE_GLOBAL: u8 = 16;
pub const OPCODE_GET_GLOBAL: u8 = 17;
pub const OPCODE_SET_GLOBAL: u8 = 18;
pub const OPCODE_GET_LOCAL: u8 = 19;
pub const OPCODE_SET_LOCAL: u8 = 20;
pub const OPCODE_JUMP_IF_FALSE: u8 = 21;
pub const OPCODE_JUMP: u8 = 22;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Constant(u8),
    Equal,
    Greater,
    Less,
    Nil,
    True,
    False,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Print,
    Return,
    Pop,
    DefineGlobal(u8),
    GetGlobal(u8),
    SetGlobal(u8),
    GetLocal(u8),
    SetLocal(u8),
    JumpIfFalse(u8, u8),
    Jump(u8, u8),
}

impl Instruction {
    pub fn stub_jump_if_false() -> Self {
        Self::JumpIfFalse(0xff, 0xff)
    }

    pub fn stub_jump() -> Self {
        Self::Jump(0xff, 0xff)
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
    pub fn as_vec(&self) -> Vec<u8> {
        match self {
            Instruction::Constant(val) => vec![OPCODE_CONSTANT, *val],
            Instruction::Equal => vec![OPCODE_EQUAL],
            Instruction::Nil => vec![OPCODE_NIL],
            Instruction::True => vec![OPCODE_TRUE],
            Instruction::False => vec![OPCODE_FALSE],
            Instruction::Negate => vec![OPCODE_NEGATE],
            Instruction::Add => vec![OPCODE_ADD],
            Instruction::Subtract => vec![OPCODE_SUBTRACT],
            Instruction::Multiply => vec![OPCODE_MULTIPLY],
            Instruction::Divide => vec![OPCODE_DIVIDE],
            Instruction::Not => vec![OPCODE_NOT],
            Instruction::Return => vec![OPCODE_RETURN],
            Instruction::Greater => vec![OPCODE_GREATER],
            Instruction::Less => vec![OPCODE_LESS],
            Instruction::Print => vec![OPCODE_PRINT],
            Instruction::Pop => vec![OPCODE_POP],
            Instruction::DefineGlobal(val) => vec![OPCODE_DEFINE_GLOBAL, *val],
            Instruction::GetGlobal(val) => vec![OPCODE_GET_GLOBAL, *val],
            Instruction::SetGlobal(val) => vec![OPCODE_SET_GLOBAL, *val],
            Instruction::GetLocal(val) => vec![OPCODE_GET_LOCAL, *val],
            Instruction::SetLocal(val) => vec![OPCODE_SET_LOCAL, *val],
            Instruction::JumpIfFalse(low, high) => vec![OPCODE_JUMP_IF_FALSE, *low, *high],
            Instruction::Jump(low, high) => vec![OPCODE_JUMP, *low, *high],
        }
    }

    pub fn fetch(buffer: &[u8], offset: &mut usize) -> FetchResult<Self> {
        let byte = consume(buffer, offset).ok_or(FetchError::End)?;
        match byte {
            OPCODE_CONSTANT => {
                let arg1 = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::Constant(arg1))
            }
            OPCODE_EQUAL => Ok(Instruction::Equal),
            OPCODE_GREATER => Ok(Instruction::Greater),
            OPCODE_LESS => Ok(Instruction::Less),
            OPCODE_NEGATE => Ok(Instruction::Negate),

            OPCODE_NIL => Ok(Instruction::Nil),
            OPCODE_TRUE => Ok(Instruction::True),
            OPCODE_FALSE => Ok(Instruction::False),

            OPCODE_ADD => Ok(Instruction::Add),
            OPCODE_SUBTRACT => Ok(Instruction::Subtract),
            OPCODE_MULTIPLY => Ok(Instruction::Multiply),
            OPCODE_DIVIDE => Ok(Instruction::Divide),

            OPCODE_NOT => Ok(Instruction::Not),

            OPCODE_PRINT => Ok(Instruction::Print),
            OPCODE_RETURN => Ok(Instruction::Return),

            OPCODE_POP => Ok(Instruction::Pop),
            OPCODE_DEFINE_GLOBAL => {
                let arg1 = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::DefineGlobal(arg1))
            }
            OPCODE_GET_GLOBAL => {
                let arg1 = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::GetGlobal(arg1))
            }
            OPCODE_SET_GLOBAL => {
                let arg1 = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::SetGlobal(arg1))
            }
            OPCODE_GET_LOCAL => {
                let arg1 = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::GetLocal(arg1))
            }
            OPCODE_SET_LOCAL => {
                let arg1 = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::SetLocal(arg1))
            }
            OPCODE_JUMP_IF_FALSE => {
                let low = consume(buffer, offset).ok_or(FetchError::Broken)?;
                let high = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::JumpIfFalse(low, high))
            }
            OPCODE_JUMP => {
                let low = consume(buffer, offset).ok_or(FetchError::Broken)?;
                let high = consume(buffer, offset).ok_or(FetchError::Broken)?;
                Ok(Instruction::Jump(low, high))
            }
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
    fn instruction_fetch_single_byte_size() {
        let data = [
            (OPCODE_NIL, Instruction::Nil),
            (OPCODE_TRUE, Instruction::True),
            (OPCODE_FALSE, Instruction::False),
            (OPCODE_NEGATE, Instruction::Negate),
            (OPCODE_ADD, Instruction::Add),
            (OPCODE_SUBTRACT, Instruction::Subtract),
            (OPCODE_MULTIPLY, Instruction::Multiply),
            (OPCODE_DIVIDE, Instruction::Divide),
            (OPCODE_RETURN, Instruction::Return),
            (OPCODE_NOT, Instruction::Not),
            (OPCODE_LESS, Instruction::Less),
            (OPCODE_GREATER, Instruction::Greater),
            (OPCODE_EQUAL, Instruction::Equal),
        ];
        let buffer = data.iter().map(|(opcode, _)| *opcode).collect::<Vec<_>>();
        let mut offset = 0;
        while offset < buffer.len() {
            let expected = &data[offset];
            let instr = Instruction::fetch(&buffer, &mut offset);
            assert!(instr.is_ok());
            let instr = instr.unwrap();
            assert_eq!(instr, expected.1);
        }
    }

    #[test]
    fn instruction_variable_parse() {
        let data = [
            ([OPCODE_DEFINE_GLOBAL, 11], Instruction::DefineGlobal(11)),
            ([OPCODE_GET_GLOBAL, 42], Instruction::GetGlobal(42)),
            ([OPCODE_SET_GLOBAL, 31], Instruction::SetGlobal(31)),
            ([OPCODE_GET_LOCAL, 58], Instruction::GetLocal(58)),
            ([OPCODE_SET_LOCAL, 6], Instruction::SetLocal(6)),
        ];
        for (inp, exp) in data.iter() {
            let mut offset = 0;
            let instr = Instruction::fetch(inp, &mut offset);
            assert!(instr.is_ok());
            let instr = instr.unwrap();
            assert_eq!(&instr, exp);
        }
    }

    #[test]
    fn instruction_jump_parse() {
        let data = [
            (
                [OPCODE_JUMP_IF_FALSE, 58, 42],
                Instruction::JumpIfFalse(58, 42),
            ),
            ([OPCODE_JUMP, 16, 103], Instruction::Jump(16, 103)),
        ];
        for (inp, exp) in data.iter() {
            let mut offset = 0;
            let instr = Instruction::fetch(inp, &mut offset);
            assert!(instr.is_ok());
            let instr = instr.unwrap();
            assert_eq!(&instr, exp);
        }
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
