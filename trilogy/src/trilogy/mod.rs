use crate::{location::Location, NativeModule};
use std::collections::HashMap;
use std::path::Path;
use trilogy_ir::ir::Module;
use trilogy_vm::{Atom, Chunk, ChunkError, Value, VirtualMachine};

mod builder;
mod load_error;
mod runtime_error;
mod trilogy_program;

pub use builder::Builder;
pub use load_error::LoadError;
pub use runtime_error::RuntimeError;
use trilogy_program::TrilogyProgram;

#[derive(Clone, Debug)]
pub struct Trilogy {
    modules: HashMap<Location, Module>,
    libraries: HashMap<Location, NativeModule>,
    entrypoint: Location,
    vm: VirtualMachine,
}

impl Trilogy {
    fn new(
        modules: HashMap<Location, Module>,
        libraries: HashMap<Location, NativeModule>,
        entrypoint: Location,
    ) -> Self {
        let mut vm = VirtualMachine::new();
        vm.set_registers(vec![Value::Unit, Value::Array(vec![].into()), Value::Unit]);
        Self {
            modules,
            libraries,
            vm,
            entrypoint,
        }
    }

    #[cfg(feature = "std")]
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, LoadError<std::io::Error>> {
        Builder::std().build_from_file(file)
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        let program = TrilogyProgram {
            modules: &self.modules,
            libraries: &self.libraries,
            entrypoint: &self.entrypoint,
        };
        self.vm
            .run(&program)
            .map_err(|error| RuntimeError { error })
    }

    pub fn compile(&self) -> Result<Chunk, ChunkError> {
        let program = TrilogyProgram {
            modules: &self.modules,
            libraries: &self.libraries,
            entrypoint: &self.entrypoint,
        };
        self.vm.compile(&program)
    }

    pub fn atom(&self, atom: &str) -> Atom {
        self.vm.atom(atom)
    }
}
