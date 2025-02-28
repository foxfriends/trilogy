use crate::location::Location;
use std::collections::HashMap;
use std::path::Path;
use trilogy_ir::ir::Module;

mod builder;
mod test_reporter;

pub use builder::{Builder, Report};
pub use test_reporter::{TestDescription, TestReporter};

#[derive(Clone, Debug)]
enum Source {
    Trilogy {
        modules: HashMap<Location, Module>,
        entrypoint: Location,
    },
}

/// An instance of the Trilogy runtime and virtual machine.
///
/// This is the entrypoint to the whole Trilogy Programming Language, by which
/// you can run Trilogy programs and embed them within larger Rust programs.
///
/// # Implementation
///
/// Whereas the [`VirtualMachine`][trilogy_vm::VirtualMachine] is the underlying
/// VM engine, this `Trilogy` instance wraps that VM in a way that is specific to
/// the Trilogy Programming Language.
#[derive(Clone, Debug)]
pub struct Trilogy {
    source: Source,
}

pub trait ModulePath {
    fn path(&self) -> Vec<&str>;
}

impl ModulePath for &str {
    fn path(&self) -> Vec<&str> {
        vec![self]
    }
}

impl ModulePath for &[&str] {
    fn path(&self) -> Vec<&str> {
        self.to_vec()
    }
}

impl Trilogy {
    fn new(source: Source) -> Self {
        Self { source }
    }

    pub fn source_entrypoint(&self) -> Option<&Location> {
        #[allow(unreachable_patterns)]
        match &self.source {
            Source::Trilogy { entrypoint, .. } => Some(entrypoint),
            _ => None,
        }
    }

    /// Loads a Trilogy program from a Trilogy source file on the local file system.
    ///
    /// A program loaded this way uses the default global `~/.trilogy/` cache directory
    /// and is provided access to the Trilogy standard library at `trilogy:std`.
    ///
    /// # Errors
    ///
    /// Returns a [`Report`][] of all errors that occur during the loading, parsing, and
    /// analysis of the source code. The report is expected to be printed to users to
    /// provide them feedback as to what is wrong with their program.
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, Report<std::io::Error>> {
        Builder::std().build_from_source(file)
    }

    /// Runs the loaded Trilogy program by evaluating `main!()`.
    ///
    /// This is equivalent to `self.call("main", vec![])`.
    #[expect(clippy::result_unit_err, reason = "This is placeholder")]
    pub fn run(&self) -> Result<trilogy_llvm::TrilogyValue, ()> {
        Ok(self.call("main", vec![]))
    }

    /// Runs the loaded Trilogy, evaluating the exported 0-arity procedure pointed to by
    /// the given path.
    ///
    /// The returned value is the exit value of the program. This value is either:
    /// * the value provided to the first `exit` statement that gets executed.
    /// * the value returned from `main!()`, if it is not `unit`
    /// * `0` if `main!()` returns `unit`
    ///
    /// # Errors
    ///
    /// If a runtime error occurs while executing this program, that error
    /// is returned. Unfortunately at this time, those errors are hard to
    /// diagnose and could be anything from a bug in the compiler to an error
    /// in the Trilogy program.
    pub fn call(
        &self,
        main: impl ModulePath,
        parameters: Vec<String>,
    ) -> trilogy_llvm::TrilogyValue {
        match &self.source {
            Source::Trilogy {
                modules,
                entrypoint,
            } => {
                let modules = modules
                    .iter()
                    .map(|(location, module)| (location.to_string(), module))
                    .collect();
                let path = main.path();
                let main_name = path.last().unwrap();
                trilogy_llvm::evaluate(modules, &entrypoint.to_string(), main_name, parameters)
            }
        }
    }

    /// Compiles a Trilogy program to LLVM assembly code, returning a single linked module as a string.
    pub fn compile(&self) -> String {
        match &self.source {
            Source::Trilogy {
                modules,
                entrypoint,
            } => {
                let modules = modules
                    .iter()
                    .map(|(location, module)| (location.to_string(), module))
                    .collect();
                trilogy_llvm::compile_to_llvm(modules, &entrypoint.to_string(), "main")
            }
        }
    }
}
