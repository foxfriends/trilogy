use crate::{location::Location, NativeModule};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use trilogy_ir::ir::Module;
use trilogy_vm::{Atom, Chunk, ChunkError, Program, Value, VirtualMachine};

mod asm_program;
mod builder;
mod load_error;
mod runtime_error;
mod trilogy_program;

use asm_program::AsmProgram;
pub use builder::Builder;
pub use load_error::LoadError;
pub use runtime_error::RuntimeError;
use trilogy_program::TrilogyProgram;

#[derive(Clone, Debug)]
enum Source {
    Trilogy {
        modules: HashMap<Location, Module>,
        entrypoint: Location,
    },
    Asm {
        asm: String,
    },
}

#[derive(Clone, Debug)]
pub struct Trilogy {
    source: Source,
    libraries: HashMap<Location, NativeModule>,
    vm: VirtualMachine,
}

impl Trilogy {
    fn new(source: Source, libraries: HashMap<Location, NativeModule>) -> Self {
        let mut vm = VirtualMachine::new();
        vm.set_registers(vec![
            // Global effect handler resume continuation
            Value::Unit,
            // Module parameters
            Value::Array(vec![].into()),
            // Query state construction bindset
            Value::Unit,
            // Local temporary
            Value::Unit,
        ]);
        Self {
            source,
            libraries,
            vm,
        }
    }

    #[cfg(feature = "std")]
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, LoadError<std::io::Error>> {
        Builder::std().build_from_source(file)
    }

    #[cfg(feature = "std")]
    pub fn from_asm(file: &mut dyn Read) -> Result<Self, std::io::Error> {
        Builder::std().build_from_asm(file)
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        let trilogy_program;
        let asm_program;
        let program: &dyn Program;
        match &self.source {
            Source::Asm { asm } => {
                asm_program = AsmProgram {
                    source: asm,
                    libraries: &self.libraries,
                };
                program = &asm_program;
            }
            Source::Trilogy {
                modules,
                entrypoint,
            } => {
                trilogy_program = TrilogyProgram {
                    libraries: &self.libraries,
                    modules: modules,
                    entrypoint: entrypoint,
                    to_asm: false,
                };
                program = &trilogy_program;
            }
        }
        self.vm.run(program).map_err(|error| RuntimeError { error })
    }

    pub fn compile(&self, debug: bool) -> Result<Chunk, ChunkError> {
        let trilogy_program;
        let asm_program;
        let program: &dyn Program;
        match &self.source {
            Source::Asm { asm } => {
                asm_program = AsmProgram {
                    source: asm,
                    libraries: &self.libraries,
                };
                program = &asm_program;
            }
            Source::Trilogy {
                modules,
                entrypoint,
            } => {
                trilogy_program = TrilogyProgram {
                    libraries: &self.libraries,
                    modules: modules,
                    entrypoint: entrypoint,
                    to_asm: !debug,
                };
                program = &trilogy_program;
            }
        }
        self.vm.compile(program)
    }

    pub fn atom(&self, atom: &str) -> Atom {
        self.vm.atom(atom)
    }
}
