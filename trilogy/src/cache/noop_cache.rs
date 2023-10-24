use super::Cache;
use crate::location::Location;

/// A Trilogy module cache that does not cache anything.
pub struct NoopCache;

impl Cache for NoopCache {
    type Error = std::convert::Infallible;

    fn has(&self, _location: &Location) -> bool {
        false
    }

    fn load(&self, _location: &Location) -> Result<String, Self::Error> {
        unimplemented!()
    }

    fn save(&self, _location: &Location, _source: &str) -> Result<(), Self::Error> {
        Ok(())
    }
}
