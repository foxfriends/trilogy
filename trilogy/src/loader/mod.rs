mod binder;
pub mod cache;
mod error;
mod linker;
#[allow(clippy::module_inception)]
mod loader;
mod location;
mod module;
mod program;
mod wip_binder;

pub use binder::Binder;
pub use cache::Cache;
pub use error::{Error, ErrorKind};
pub use linker::LinkerError;
pub use loader::Loader;
pub use location::Location;
pub use module::Module;
pub use program::Program;

pub type Result<T> = std::result::Result<T, Error>;
