#[cfg(feature = "std")]
mod stdlib;

use trilogy_loader::{LinkerError, Loader};
use trilogy_parser::syntax::SyntaxError;
pub use trilogy_vm::runtime::*;
use trilogy_vm::{Program, VirtualMachine};

#[cfg(feature = "derive")]
pub use trilogy_derive::*;

use std::fmt::{self, Display};
use std::path::Path;

pub struct Trilogy {
    vm: VirtualMachine,
}

pub enum LoadError {
    SyntaxError(Vec<SyntaxError>),
    LinkerError(Vec<LinkerError>),
}

#[derive(Debug)]
pub struct RuntimeError<'a> {
    program: &'a Program,
    error: trilogy_vm::Error,
}

impl std::error::Error for RuntimeError<'_> {}

impl Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Trace:\n{}", self.error.trace(self.program))?;
        write!(f, "Dump:\n{}", self.error.dump())?;
        write!(f, "{}", self.error)?;
        Ok(())
    }
}

impl Trilogy {
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, LoadError> {
        let loader = Loader::new(file.as_ref().to_owned());
        let binder = loader.load().unwrap();
        if binder.has_errors() {
            return Err(LoadError::SyntaxError(binder.errors().cloned().collect()));
        }
        let program = match binder.analyze() {
            Ok(program) => program,
            Err(errors) => return Err(LoadError::LinkerError(errors)),
        };
        let program = program.generate_code();
        Ok(Self::from(program))
    }

    pub fn run<'a>(&'a mut self) -> Result<Value, RuntimeError<'a>> {
        self.vm.run().map_err(|error| RuntimeError {
            program: self.vm.program(),
            error,
        })
    }
}

impl From<Program> for Trilogy {
    fn from(program: Program) -> Self {
        let vm = VirtualMachine::load(program);
        Self { vm }
    }
}
