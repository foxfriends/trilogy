use crate::{location::Location, NativeModule};
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
        let vm = VirtualMachine::new();
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
        let mut program = TrilogyProgram {
            modules: &self.modules,
            libraries: &self.libraries,
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
    libraries: &'a HashMap<Location, NativeModule>,
    entrypoint: &'a Location,
}

impl Program for TrilogyProgram<'_> {
    fn entrypoint(&mut self, mut chunk: ChunkBuilder) -> Chunk {
        let module = self.modules.get(self.entrypoint).unwrap();
        trilogy_codegen::write_program(&mut chunk, module);
        chunk.build().unwrap()
    }

    fn chunk(&mut self, input: Value, mut chunk: ChunkBuilder) -> Chunk {
        let location = match input {
            Value::String(url) => Location::absolute(url.parse().expect("invalid module location")),
            _ => panic!("invalid module specifier `{input}`"),
        };
        enum Either<'a> {
            Source(&'a Module),
            Native(&'a NativeModule),
        }
        let module = self
            .modules
            .get(&location)
            .map(Either::Source)
            .or_else(|| self.libraries.get(&location).map(Either::Native))
            .expect("unknown module location");
        match module {
            Either::Source(module) => trilogy_codegen::write_module(&mut chunk, module),
            Either::Native(..) => todo!("native modules"),
        }
        chunk.build().unwrap()
    }
}
