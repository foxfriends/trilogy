#[allow(dead_code)]
mod asm;
pub(crate) mod chunk;
mod instruction;
mod optimization;

pub use chunk::{Chunk, ChunkBuilder, ChunkError, ChunkWriter};
pub use instruction::{Instruction, Offset, OpCode, OpCodeError};
