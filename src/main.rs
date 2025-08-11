use crate::{chunk::Chunk, vm::Machine};

mod chunk;
mod vm;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write_opcode(vm::OpCode::Constant, 123);
    chunk.write_u8(constant as u8, 123);
    chunk.write_opcode(vm::OpCode::Return, 123);

    let mut machine = Machine::with(chunk);
    _ = machine.run();
}
