use std::convert::Infallible;
use std::fmt::{self, Display};
use std::path::Path;
use trilogy_vm::{Program, Value, VirtualMachine};

mod builder;
mod load_error;
mod runtime_error;

pub use builder::Builder;
pub use load_error::LoadError;
pub use runtime_error::RuntimeError;

pub struct Trilogy {
    vm: VirtualMachine,
}

impl Trilogy {
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, LoadError<Infallible>> {
        Builder::default().build_from_file(file)
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
