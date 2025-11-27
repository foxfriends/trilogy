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
//! In the simplest case, a Trilogy program is loaded from a file external to the
//! running Rust program.
//!
//! ```no_run
//! use trilogy::Trilogy;
//! let trilogy = Trilogy::from_file("./path/to/main.tri").unwrap();
//! let exit_value = trilogy.run().unwrap();
//! ```
//!
//! For more advanced usage, the [`Builder`][] allows for customizing the module
//! resolution system and injecting libraries directly into the instance.
//!
//! ```no_run
//! use trilogy::Builder;
//! let trilogy = Builder::new().build_from_source("./path/to/main.tri").unwrap();
//! let exit_value = trilogy.run().unwrap();
//! ```

#[path = "stdlib/mod.rs"]
mod stdlib;

mod ariadne;
mod cache;
mod location;
pub(crate) mod trilogy;

pub use cache::{Cache, FileSystemCache, NoopCache};
pub use location::Location;
pub use trilogy::{Builder, Report, Trilogy};
