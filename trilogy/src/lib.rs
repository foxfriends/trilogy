//! The Rust interface to the Trilogy Programming Language, allowing Trilogy
//! programs to be embedded in Rust programs, as well as allowing Rust
//! programs to extend the functionality of the Trilogy programming language
//! with native capabilities.
//!
//! Trilogy also provides a command line interface by which pure Trilogy programs
//! can be run, with access to the Trilogy standard library.
//!
//! # Embedding
//!

#![cfg_attr(
    feature = "std",
    doc = r##"
In the simplest case, a Trilogy program is loaded from a file external to the
running Rust program.

```no_run
use trilogy::Trilogy;
let trilogy = Trilogy::from_file("./path/to/main.tri").unwrap();
let exit_value = trilogy.run().unwrap();
```

For more advanced usage, the [`Builder`][] allows for customizing the module
resolution system and injecting libraries directly into the instance.

```no_run
use trilogy::Builder;
let trilogy = Builder::new().build_from_source("./path/to/main.tri").unwrap();
let exit_value = trilogy.run().unwrap();
```
"##
)]
#![cfg_attr(
    not(feature = "std"),
    doc = r##"
Without the `std` feature enabled, the [`Builder`][] must be used to manually set up the
module resolution system and injected libraries to compile the program.

```no_run
use trilogy::Builder;
let trilogy = Builder::new().build_from_source("./path/to/main.tri").unwrap();
let exit_value = trilogy.run().unwrap();
```
"##
)]

#[cfg(feature = "std")]
#[cfg_attr(all(feature = "llvm"), path = "stdlib-llvm/mod.rs")]
mod stdlib;

#[cfg(feature = "macros")]
pub use trilogy_derive::*;

#[cfg(feature = "tvm")]
pub use trilogy_vm::runtime::*;

mod ariadne;
mod cache;
mod location;
#[cfg(feature = "tvm")]
mod runtime;
pub(crate) mod trilogy;

pub use cache::{Cache, FileSystemCache, NoopCache};
pub use location::Location;
#[cfg(feature = "tvm")]
pub use runtime::{
    NativeMethod, NativeMethodFn, NativeModule, NativeModuleBuilder, NativeType, NativeTypeBuilder,
    Runtime, RuntimeError,
};
pub use trilogy::{Builder, Report, TestDescription, TestReporter, Trilogy};

/// The result type to use for native functions.
#[cfg(feature = "tvm")]
pub type Result<T> = std::result::Result<T, trilogy_vm::Error>;
