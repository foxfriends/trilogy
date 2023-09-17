#[allow(dead_code)]
mod asm;
mod chunk;
mod instruction;

pub use chunk::{Chunk, ChunkBuilder, ChunkError};
pub use instruction::{Instruction, Offset, OpCode};
