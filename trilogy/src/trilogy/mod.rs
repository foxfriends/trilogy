use crate::{location::Location, NativeModule};
use std::collections::HashMap;
use std::path::Path;
use trilogy_ir::ir::Module;
use trilogy_vm::{Atom, Chunk, ChunkError, Value, VirtualMachine};

mod asm_program;
mod builder;
mod runtime;
mod runtime_error;
mod test_reporter;
mod trilogy_program;
mod trilogy_test;

pub use builder::{Builder, Report};
pub use runtime::Runtime;
pub use runtime_error::RuntimeError;
pub use test_reporter::{TestDescription, TestReporter};

use asm_program::AsmProgram;
use trilogy_program::TrilogyProgram;
use trilogy_test::TrilogyTest;

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

    fn default_registers() -> Vec<Value> {
        vec![
            // Global effect handler resume continuation
            Value::Unit,
            // Module parameters
            Value::Array(vec![].into()),
            // Query state construction bindset
            Value::Unit,
            // Local temporary
            Value::Unit,
        ]
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
    #[cfg(feature = "std")]
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self, Report<std::io::Error>> {
        Builder::std().build_from_source(file)
    }

    /// Runs the loaded Trilogy program by evaluating `main!()`.
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
    pub fn run(&self) -> Result<Value, RuntimeError> {
        let result = match &self.source {
            Source::Asm { asm } => self.vm.run_with_registers(
                &AsmProgram {
                    source: asm,
                    libraries: &self.libraries,
                },
                Self::default_registers(),
            ),
            Source::Trilogy {
                modules,
                entrypoint,
            } => self.vm.run_with_registers(
                &TrilogyProgram {
                    libraries: &self.libraries,
                    modules,
                    entrypoint,
                    path: &["main"],
                    to_asm: false,
                },
                Self::default_registers(),
            ),
        };
        result.map_err(|er| er.into())
    }

    /// Runs all tests found within the program.
    ///
    /// This only includes tests that are found within the user's program and not any tests
    /// found in libraries. Libraries are expected to have tested themselves already before
    /// being published as a library.
    ///
    /// For each test, the program is compiled as if that test were `main`, and then
    /// called. If a test function runs to completion, it is considered a success, otherwise
    /// it is a failure which is added to the test report. The test report is not yet implemented.
    pub fn run_tests(&self, reporter: &mut dyn TestReporter) {
        use trilogy_ir::ir::{DefinitionItem, TestDefinition};

        fn locate_tests(module: &Module) -> impl Iterator<Item = (Vec<&str>, &TestDefinition)> {
            module.definitions().iter().flat_map(
                |def| -> Box<dyn Iterator<Item = (Vec<&str>, &TestDefinition)>> {
                    match &def.item {
                        DefinitionItem::Test(def) => {
                            Box::new(std::iter::once((vec![], def.as_ref())))
                        }
                        DefinitionItem::Module(module) => Box::new(
                            module
                                .module
                                .as_module()
                                .into_iter()
                                .flat_map(locate_tests)
                                .map(|(mut path, test)| {
                                    path.insert(0, module.name.id.name().unwrap());
                                    (path, test)
                                }),
                        ),
                        _ => Box::new(std::iter::empty()),
                    }
                },
            )
        }

        reporter.begin();

        if let Source::Trilogy { modules, .. } = &self.source {
            let tests = modules.iter().flat_map(|(location, module)| {
                locate_tests(module).map(|(path, test)| (location.clone(), path, test))
            });

            let mut current_location = None;
            let mut current_path = vec![];

            for (location, path, test) in tests {
                match current_location {
                    None => {
                        reporter.enter_document(&location);
                        current_location = Some(location.clone());
                    }
                    Some(loc) if loc != location => {
                        reporter.exit_document();
                        reporter.enter_document(&location);
                        current_location = Some(location.clone());
                    }
                    _ => {}
                }

                while !path.starts_with(&current_path) {
                    current_path.pop();
                    reporter.exit_module();
                }
                while !current_path.starts_with(&path) {
                    let seg = path.get(current_path.len()).unwrap();
                    reporter.enter_module(seg);
                    current_path.push(seg);
                }

                let result = self.vm.run_with_registers(
                    &TrilogyTest {
                        libraries: &self.libraries,
                        modules,
                        entrypoint: &location,
                        path: &path,
                        test: &test.name,
                        to_asm: false,
                    },
                    Self::default_registers(),
                );
                reporter.test_result(
                    &test.name,
                    TestDescription {
                        negated: test.negated,
                    },
                    result,
                );
            }
        }

        reporter.finish();
    }

    /// Compiles a Trilogy program to bytecode, returning the compiled program as a Chunk.
    ///
    /// That chunk may be saved to a file and later loaded back into Trilogy using `Trilogy::from_asm`.
    ///
    /// # Errors
    ///
    /// Returns an error if the program bytecode generation fails for any reason. That reason is
    /// likely a bug in the compiler, as a program that has been successfully parsed and checked
    /// up to this point should be able to be compiled.
    pub fn compile(&self) -> Result<Chunk, ChunkError> {
        match &self.source {
            Source::Asm { asm } => self.vm.compile(&AsmProgram {
                source: asm,
                libraries: &self.libraries,
            }),
            Source::Trilogy {
                modules,
                entrypoint,
            } => self.vm.compile(&TrilogyProgram {
                libraries: &self.libraries,
                modules,
                entrypoint,
                path: &["main"],
                to_asm: true,
            }),
        }
    }

    #[doc(hidden)]
    pub fn compile_debug(&self) -> Result<Chunk, ChunkError> {
        match &self.source {
            Source::Asm { asm } => self.vm.compile(&AsmProgram {
                source: asm,
                libraries: &self.libraries,
            }),
            Source::Trilogy {
                modules,
                entrypoint,
            } => self.vm.compile(&TrilogyProgram {
                libraries: &self.libraries,
                modules,
                entrypoint,
                path: &["main"],
                to_asm: false,
            }),
        }
    }

    /// Creates an atom in the context of this Trilogy engine, in the same way that atom
    /// literals are created within the Trilogy program.
    ///
    /// See [`Atom`][] for more details.
    pub fn atom(&self, atom: &str) -> Atom {
        self.vm.atom(atom)
    }

    /// Creates a new, anonymous atom. This atom has never been created before, and can
    /// never be created again, even other atoms with the same tag will not be equivalent.
    ///
    /// See [`Atom`][] for more details.
    pub fn atom_anon(&self, atom: &str) -> Atom {
        self.vm.atom_anon(atom)
    }
}
