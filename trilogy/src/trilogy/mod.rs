use crate::location::Location;
use crate::NativeModule;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use trilogy_ir::ir::Module;
use trilogy_vm::{Atom, Chunk, ChunkError, Program, Value, VirtualMachine};

mod asm_program;
pub mod builder;
mod runtime;
mod runtime_error;
mod trilogy_program;

use asm_program::AsmProgram;
use builder::{Builder, Report};
pub use runtime::Runtime;
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
        Self {
            source,
            libraries,
            vm: VirtualMachine::new(),
        }
    }

    #[cfg(feature = "std")]
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, Report<std::io::Error>> {
        Builder::std().build_from_source(file)
    }

    #[cfg(feature = "std")]
    pub fn from_asm(file: &mut dyn Read) -> Result<Self, std::io::Error> {
        Builder::std().build_from_asm(file)
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        let registers = vec![
            // Global effect handler resume continuation
            Value::Unit,
            // Module parameters
            Value::Array(vec![].into()),
            // Query state construction bindset
            Value::Unit,
            // Local temporary
            Value::Unit,
        ];
        let result = match &self.source {
            Source::Asm { asm } => self.vm.run_with_registers(
                &AsmProgram {
                    source: asm,
                    libraries: &self.libraries,
                },
                registers,
            ),
            Source::Trilogy {
                modules,
                entrypoint,
            } => self.vm.run_with_registers(
                &TrilogyProgram {
                    libraries: &self.libraries,
                    modules,
                    entrypoint,
                    to_asm: false,
                },
                registers,
            ),
        };
        result.map_err(|er| er.into())
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
                    modules,
                    entrypoint,
                    to_asm: !debug,
                };
                program = &trilogy_program;
            }
        }
        self.vm.compile(program)
    }

    /// Creates an atom in the context of this Trilogy engine, in the same way that atom
    /// literals are created within the Trilogy program.
    pub fn atom(&self, atom: &str) -> Atom {
        self.vm.atom(atom)
    }

    /// Creates a new, anonymous atom. This atom has never been created before, and can
    /// never be created again, even other atoms with the same tag will not be equivalent.
    pub fn atom_anon(&self, atom: &str) -> Atom {
        self.vm.atom_anon(atom)
    }
}
