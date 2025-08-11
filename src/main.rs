use crate::chunk::Chunk;

mod chunk;
mod vm;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write_opcode(vm::OpCode::Constant, 123);
    chunk.write_u8(constant as u8, 123);
    chunk.write_opcode(vm::OpCode::Return, 123);

    let disassm = chunk.disassemble();
    println!("{disassm}")
}
