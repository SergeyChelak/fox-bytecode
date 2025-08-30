mod chunk;
mod func;
mod instruction;
mod upvalue_data;
mod value;

pub use chunk::Chunk;
pub use func::*;
pub use instruction::*;
pub use upvalue_data::*;
pub use value::{OperationError, Value, ValueOperation};

pub const UINT8_COUNT: usize = 256;
pub const MAX_FUNCTION_ARGUMENTS: usize = 255;

fn consume_byte(buffer: &[u8], offset: &mut usize) -> Option<u8> {
    let byte = buffer.get(*offset)?;
    *offset += 1;
    Some(*byte)
}
