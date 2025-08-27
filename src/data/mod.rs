mod chunk;
mod func;
mod instruction;
mod value;

pub use chunk::Chunk;
pub use func::*;
pub use instruction::*;
pub use value::{OperationError, Value, ValueOperation};

pub const UINT8_COUNT: usize = 256;
pub const MAX_FUNCTION_ARGUMENTS: usize = 255;
