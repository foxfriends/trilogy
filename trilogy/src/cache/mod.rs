use crate::location::Location;

mod file_system_cache;
mod noop_cache;

pub use file_system_cache::FileSystemCache;
pub use noop_cache::NoopCache;

/// A type that can be used as a cache loading Trilogy modules during compilation.
pub trait Cache {
    /// An error that occurs when loading or saving to this cache.
    type Error: std::error::Error + 'static;

    /// Does this cache store the module at this Location?
    fn has(&self, location: &Location) -> bool;

    /// Loads the cached source code for the module at the Location.
    ///
    /// Will only be called for locations for which `has` returns `true`.
    fn load(&self, location: &Location) -> Result<String, Self::Error>;

    /// Saves the source associated with the location for later access.
    fn save(&self, location: &Location, source: &str) -> Result<(), Self::Error>;
}
