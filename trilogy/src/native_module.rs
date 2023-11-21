use crate::Runtime;
use std::collections::HashMap;
use trilogy_vm::{Error, Execution, Native, NativeFunction, Value};

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
    pub(crate) items: HashMap<&'static str, Native>,
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
                items: Default::default(),
            },
        }
    }
}

impl NativeModuleBuilder {
    /// Create a new empty module builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a native procedure or module to this module under a given name.
    ///
    /// Native functions are typically created using the [`#[proc]`][trilogy_derive::proc]
    /// or [`#[module]`][trilogy_derive::module] attribute macros.
    ///
    /// The provided name will be used to reference the procedure from a Trilogy program
    /// so it must be a valid name for use as an identifier in Trilogy, otherwise it will
    /// not be usable.
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
    ///     .add_item("hello", hello)
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
    pub fn add_item<N: NativeFunction + 'static>(mut self, name: &'static str, proc: N) -> Self {
        self.inner.items.insert(name, proc.into());
        self
    }

    /// Finish building this native module.
    pub fn build(self) -> NativeModule {
        self.inner
    }
}

impl NativeFunction for NativeModule {
    fn arity(&self) -> usize {
        2 // the symbol + the module key
    }

    fn call(&mut self, ex: &mut Execution, input: Vec<Value>) -> Result<(), Error> {
        let runtime = Runtime::new(ex);
        let atom = runtime.unlock_module(input)?;
        if let Some(proc) = self.items.get(atom.as_ref()) {
            return runtime.r#return(proc.clone());
        }

        let symbol_list = self
            .items
            .keys()
            .map(|name| Value::from(runtime.atom(name)))
            .collect::<Vec<_>>();
        Err(runtime.unresolved_import(atom, symbol_list))
    }
}
