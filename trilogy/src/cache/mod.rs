use crate::location::Location;

mod file_system_cache;
mod noop_cache;

pub use file_system_cache::FileSystemCache;
pub use noop_cache::NoopCache;

pub trait Cache {
    type Error: std::error::Error + 'static;

    fn has(&self, location: &Location) -> bool;
    fn load(&self, location: &Location) -> Result<String, Self::Error>;
    fn save(&self, location: &Location, source: &str) -> Result<(), Self::Error>;
}
