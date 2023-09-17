mod traits {
    pub(crate) trait Tags {
        type Tag;
        fn tag(&self) -> Self::Tag;
    }
}

mod bytecode;
mod cactus;
pub mod runtime;
mod vm;

pub use bytecode::{Chunk, ChunkBuilder, Instruction, Offset, OpCode};
pub use runtime::*;
pub use vm::{Error, ErrorKind, Program, VirtualMachine};
