//! Bridges the runtime of the Trilogy Virtual Machine to the host program.

use crate::{Chunk, VirtualMachine};

mod array;
pub(crate) mod atom;
mod bits;
pub(crate) mod callable;
mod eq;
mod number;
mod record;
mod set;
mod r#struct;
mod tuple;
mod value;

pub use array::Array;
pub use atom::Atom;
pub use bits::Bits;
pub use callable::{Callable, Native, NativeFunction};
pub use eq::{ReferentialEq, StructuralEq};
pub use number::Number;
pub use r#struct::Struct;
pub use record::Record;
pub use set::Set;
pub use tuple::Tuple;
pub use value::Value;

pub struct Runtime<'a> {
    vm: &'a mut VirtualMachine,
}

impl<'a> Runtime<'a> {
    pub(crate) fn new(vm: &'a mut VirtualMachine) -> Runtime<'a> {
        Runtime { vm }
    }

    pub fn asm(&self, _chunk: Chunk) -> impl Iterator<Item = Value> {
        std::iter::empty()
    }

    pub fn atom(&self, tag: &str) -> Atom {
        self.vm.atom(tag)
    }

    pub fn atom_anon(&self, tag: &str) -> Atom {
        self.vm.atom_anon(tag)
    }
}
