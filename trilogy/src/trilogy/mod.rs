use crate::location::Location;
use std::collections::HashMap;
use std::path::Path;
use trilogy_ir::ir::Module;
use trilogy_vm::{Atom, Chunk, ChunkBuilder, Program, Value, VirtualMachine};

mod builder;
mod load_error;
mod runtime_error;

pub use builder::Builder;
pub use load_error::LoadError;
pub use runtime_error::RuntimeError;

pub struct Trilogy {
    modules: HashMap<Location, Module>,
    entrypoint: Location,
    vm: VirtualMachine,
}

impl Trilogy {
    fn new(modules: HashMap<Location, Module>, entrypoint: Location) -> Self {
        let vm = VirtualMachine::new();
        Self {
            modules,
            vm,
            entrypoint,
        }
    }

    #[cfg(feature = "std")]
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, LoadError<std::io::Error>> {
        Builder::std().build_from_file(file)
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        let mut program = TrilogyProgram {
            modules: &self.modules,
            entrypoint: &self.entrypoint,
        };
        self.vm
            .run(&mut program)
            .map_err(|error| RuntimeError { error })
    }

    pub fn atom(&self, atom: &str) -> Atom {
        self.vm.atom(atom)
    }
}

struct TrilogyProgram<'a> {
    modules: &'a HashMap<Location, Module>,
    entrypoint: &'a Location,
}

impl Program for TrilogyProgram<'_> {
    fn entrypoint(&mut self, mut chunk: ChunkBuilder) -> Chunk {
        let module = self.modules.get(self.entrypoint).unwrap();
        chunk.jump("main");
        trilogy_codegen::write_program(&mut chunk, module);
        chunk.build().unwrap()
    }

    fn chunk(&mut self, _input: Value, _chunk: ChunkBuilder) -> Chunk {
        todo!()
    }
}
