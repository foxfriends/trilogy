//! The Trilogy Virtual Machine.
//!
//! This virtual machine, though designed specifically for Trilogy, is built so as to
//! be reusable by any language which compiles to its bytecode and provides a suitable
//! chunk resolution system.

mod bytecode;
mod cactus;
pub mod runtime;
mod vm;

pub use bytecode::{
    Annotation, Chunk, ChunkBuilder, ChunkError, ChunkWriter, Instruction, Location, Note, Offset,
    OpCode,
};
pub use runtime::*;
pub use vm::{Error, ErrorKind, Execution, InternalRuntimeError, Program, VirtualMachine};
