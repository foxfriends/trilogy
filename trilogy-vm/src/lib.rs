mod traits {
    pub(crate) trait Tags {
        type Tag;
        fn tag(&self) -> Self::Tag;
    }
}

mod bytecode;
mod cactus;
mod program;
pub mod runtime;
mod vm;

pub use bytecode::{AsmError, Instruction, LabelAlreadyInserted, OpCode};
pub use program::{Program, ProgramBuilder};
pub use runtime::*;
pub use vm::VirtualMachine;
