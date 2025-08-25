mod chunk;
mod func;
mod instruction;
mod value;

pub use chunk::Chunk;
pub use func::*;
pub use instruction::*;
pub use value::{OperationError, Value, ValueOperation};
