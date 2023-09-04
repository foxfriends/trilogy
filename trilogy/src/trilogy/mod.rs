use crate::loader::LinkerError;
use std::convert::Infallible;
use std::fmt::{self, Display};
use std::path::Path;
use trilogy_parser::syntax::SyntaxError;
use trilogy_vm::{Program, Value, VirtualMachine};

mod builder;

pub use builder::Builder;

#[derive(Debug)]
pub enum LoadError<E: std::error::Error> {
    InvalidScheme(String),
    Cache(E),
    Syntax(Vec<SyntaxError>),
    Linker(Vec<LinkerError>),
    External(Box<dyn std::error::Error>),
}

impl<E: std::error::Error> LoadError<E> {
    pub(crate) fn external(error: impl std::error::Error + 'static) -> Self {
        Self::External(Box::new(error))
    }
}

#[derive(Debug)]
pub struct RuntimeError<'a> {
    program: &'a Program,
    error: trilogy_vm::Error,
}

impl<E: std::error::Error> std::error::Error for LoadError<E> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::External(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl<E: std::error::Error> Display for LoadError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cache(error) => write!(f, "{error}"),
            Self::InvalidScheme(scheme) => {
                write!(f, "invalid scheme in module location `{}`", scheme)
            }
            Self::Syntax(errors) => {
                for error in errors {
                    writeln!(f, "{error:#?}")?;
                }
                Ok(())
            }
            Self::Linker(errors) => {
                for error in errors {
                    writeln!(f, "{error:#?}")?;
                }
                Ok(())
            }
            Self::External(error) => write!(f, "{error}"),
        }
    }
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
