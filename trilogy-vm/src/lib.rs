#![allow(dead_code)]

mod bytecode;
mod cactus;
mod runtime;
mod vm;

pub use bytecode::Instruction;
pub use runtime::{Array, Atom, Record, ReferentialEq, Set, Struct, StructuralEq, Tuple, Value};
pub use vm::VirtualMachine;
