#[cfg(feature = "std")]
mod stdlib;
#[cfg(feature = "derive")]
pub use trilogy_derive::*;

// TODO: loader was moved here, needs cleanup later
mod loader;

use trilogy_parser::syntax::SyntaxError;
pub use trilogy_vm::runtime::*;
use trilogy_vm::{Program, VirtualMachine};

mod native_module;

use loader::LinkerError;
pub use loader::Loader;
pub use native_module::{NativeModule, NativeModuleBuilder};

use std::collections::HashMap;
use std::fmt::{self, Display};
use std::path::Path;

pub struct TrilogyBuilder {
    libraries: HashMap<&'static str, NativeModule>,
}

impl Default for TrilogyBuilder {
    fn default() -> Self {
        let mut builder = Self::new();
        #[cfg(feature = "std")]
        {
            builder = builder.library("std", stdlib::std());
        }
        builder
    }
}

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

impl TrilogyBuilder {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }

    pub fn library(mut self, name: &'static str, library: NativeModule) -> Self {
        self.libraries.insert(name, library);
        self
    }

    fn build_from_file(self, file: impl AsRef<Path>) -> Result<Trilogy, LoadError> {
        let loader = Loader::new(file.as_ref().to_owned());
        let binder = loader.load().unwrap();
        if binder.has_errors() {
            return Err(LoadError::SyntaxError(binder.errors().cloned().collect()));
        }
        let program = match binder.analyze(&self.libraries) {
            Ok(program) => program,
            Err(errors) => return Err(LoadError::LinkerError(errors)),
        };
        let program = program.generate_code();
        Ok(Trilogy::from(program))
    }
}

impl Trilogy {
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, LoadError> {
        TrilogyBuilder::default().build_from_file(file)
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError<'_>> {
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

impl Display for Trilogy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.vm.program().fmt(f)
    }
}
