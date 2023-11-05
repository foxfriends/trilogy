#[cfg(feature = "std")]
mod stdlib;
#[cfg(feature = "macros")]
pub use trilogy_derive::*;

pub use trilogy_vm::runtime::*;

mod cache;
mod location;
mod native_module;
pub(crate) mod trilogy;

pub use cache::{Cache, FileSystemCache, NoopCache};
pub use location::Location;
pub use native_module::{NativeModule, NativeModuleBuilder};
pub use trilogy::builder::Report;
pub use trilogy::{Runtime, RuntimeError, TestReporter, Trilogy};

/// The result type to use for native functions.
pub type Result<T> = std::result::Result<T, trilogy_vm::Error>;
