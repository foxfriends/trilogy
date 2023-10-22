use crate::{Atom, Chunk, Value, VirtualMachine};

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
