#[allow(dead_code)]
mod asm;
pub(crate) mod chunk;
mod instruction;

pub use chunk::{ChunkBuilder, ChunkError};
pub use instruction::{Instruction, Offset, OpCode};
