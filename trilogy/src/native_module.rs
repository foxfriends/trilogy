use crate::location::Location;
use std::collections::HashMap;
use trilogy_codegen::{INCORRECT_ARITY, INVALID_CALL, RETURN};
use trilogy_vm::{ChunkBuilder, ChunkWriter, Instruction, Native, NativeFunction, Value};

/// A module of native functions.
///
/// Native modules are provided to Trilogy at build time, allowing native Rust functions
/// to be imported into Trilogy programs by referencing them through an imported module.
///
/// Native modules themselves do not have names, but are installed into other modules
/// with names, or into the Trilogy runtime at a module location.
///
/// It is unlikely (and not recommended) to create a native module manually.
/// More likely one will be created by using the [`#[module]`][trilogy_derive::module]
/// proc macro to create a `NativeModule` from a Rust module.
#[derive(Clone, Debug)]
pub struct NativeModule {
    pub(crate) modules: HashMap<&'static str, NativeModule>,
    pub(crate) procedures: HashMap<&'static str, Native>,
}

/// Builder for native modules.
#[derive(Clone)]
pub struct NativeModuleBuilder {
    inner: NativeModule,
}

impl Default for NativeModuleBuilder {
    fn default() -> Self {
        Self {
            inner: NativeModule {
                modules: Default::default(),
                procedures: Default::default(),
            },
        }
    }
}

impl NativeModuleBuilder {
    /// Create a new empty module builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a native module as a submodule to this module.
    ///
    /// The provided name will be used to reference the submodule from a Trilogy program
    /// so it must be a valid name for use as an identifier in Trilogy, otherwise it will
    /// not be referenceable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy::NativeModuleBuilder;
    /// # let some_submodule = NativeModuleBuilder::new().build();
    /// let native_module = NativeModuleBuilder::new()
    ///     .add_submodule("sub", some_submodule)
    ///     .build();
    /// ```
    ///
    /// Installing `native_module` at the location `Location::library("module")`, the submodule
    /// could be referenced from a Trilogy program as follows:
    ///
    /// ```trilogy
    /// module native from "trilogy:module"
    /// proc main!() {
    ///     let submodule = native::sub
    /// }
    /// ```
    pub fn add_submodule(mut self, name: &'static str, module: NativeModule) -> Self {
        self.inner.modules.insert(name, module);
        self
    }

    /// Add a native function as a procedure to this module with the provided name.
    ///
    /// Native functions are typically created using the [`#[proc]`][trilogy_derive::proc]
    /// attribute macro.
    ///
    /// The provided name will be used to reference the procedure from a Trilogy program
    /// so it must be a valid name for use as an identifier in Trilogy, otherwise it will
    /// not be referenceable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy::{NativeModuleBuilder, proc, Runtime};
    /// #[proc]
    /// fn hello(rt: Runtime) -> trilogy::Result<()> {
    ///     rt.r#return("hello")
    /// }
    ///
    /// let native_module = NativeModuleBuilder::new()
    ///     .add_procedure("hello", hello)
    ///     .build();
    /// ```
    ///
    /// Installing `native_module` at the location `Location::library("module")`, the `hello`
    /// procedure could be called as follows:
    ///
    /// ```trilogy
    /// module native from "trilogy:module"
    /// proc main!() {
    ///     let hello = native::hello!()
    /// }
    /// ```
    pub fn add_procedure<N: NativeFunction + 'static>(
        mut self,
        name: &'static str,
        proc: N,
    ) -> Self {
        self.inner.procedures.insert(name, proc.into());
        self
    }

    /// Finish building this native module.
    pub fn build(self) -> NativeModule {
        self.inner
    }
}

impl NativeModule {
    pub(crate) fn write_to_chunk(&self, location: &Location, chunk: &mut ChunkBuilder) {
        chunk.close(RETURN);
        self.write_to_chunk_at_path(location, vec![], chunk)
    }

    fn write_to_chunk_at_path(
        &self,
        location: &Location,
        path: Vec<&str>,
        chunk: &mut ChunkBuilder,
    ) {
        let pathstr = path.iter().fold(String::new(), |s, seg| s + seg + "::");
        chunk
            .instruction(Instruction::Destruct)
            .instruction(Instruction::Copy)
            .atom("module")
            .instruction(Instruction::ValEq)
            .cond_jump(INVALID_CALL)
            .instruction(Instruction::Pop)
            .instruction(Instruction::Copy)
            .constant(1)
            .instruction(Instruction::ValEq)
            .cond_jump(INCORRECT_ARITY)
            .instruction(Instruction::Pop);
        for (name, proc) in &self.procedures {
            let next = format!("#skip::{location}::{pathstr}{name}");
            chunk
                .instruction(Instruction::Copy)
                .atom(name)
                .instruction(Instruction::ValEq)
                .cond_jump(&next)
                .instruction(Instruction::Pop)
                .constant(proc.clone())
                .instruction(Instruction::Return)
                .label(next);
        }
        for (name, module) in &self.modules {
            let next = format!("#skip::{location}::{pathstr}{name}");
            let module_label = format!("{location}::{pathstr}{name}");
            chunk
                .instruction(Instruction::Copy)
                .atom(name)
                .instruction(Instruction::ValEq)
                .cond_jump(&next)
                .instruction(Instruction::Pop)
                .reference(&module_label)
                .instruction(Instruction::Return)
                .label(module_label);
            let mut child_path = path.clone();
            child_path.push(name);
            module.write_to_chunk_at_path(location, child_path, chunk);
            chunk.label(next);
        }
        let symbol_list = self
            .procedures
            .keys()
            .chain(self.modules.keys())
            .map(|name| Value::from(chunk.make_atom(name)))
            .collect::<Vec<_>>();
        chunk
            .constant(symbol_list)
            .instruction(Instruction::Cons)
            .atom("UnresolvedImport")
            .atom("UnresolvedImport")
            .instruction(Instruction::Construct)
            .instruction(Instruction::Panic);
    }
}
