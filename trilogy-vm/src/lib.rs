mod traits {
    pub(crate) trait Tags {
        type Tag;
        fn tag(&self) -> Self::Tag;
    }
}

mod bytecode;
mod cactus;
mod program;
mod runtime;
mod vm;

pub use bytecode::{AsmError, Instruction, LabelAlreadyInserted};
pub use program::{Program, ProgramBuilder};
pub use runtime::{
    Array, Atom, Bits, Continuation, Number, Record, ReferentialEq, Set, Struct, StructuralEq,
    Tuple, Value,
};
pub use vm::VirtualMachine;
