use crate::cache::{Cache, NoopCache};
use crate::wip_binder::WipBinder;
use crate::{Binder, Error, ErrorKind, Location};
use std::path::PathBuf;
use trilogy_parser::syntax::Document;
use trilogy_parser::Parse;

pub struct Loader<C> {
    base_path: PathBuf,
    cache: C,
}

impl Loader<NoopCache> {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            cache: NoopCache,
        }
    }
}

impl<T> Loader<T> {
    pub fn cache<C>(self, cache: C) -> Loader<C> {
        Loader {
            base_path: self.base_path,
            cache,
        }
    }
}

impl<C: Cache> Loader<C>
where
    C::Error: std::error::Error + 'static,
{
    /// Load the modules described by this `Loader`.
    ///
    /// Modules loaded are cached at the cache directory, which defaults to a
    /// `.trilogy-cache` relative to the provided base path.
    ///
    /// A returned `Err` here refers to whether the loading process could be completed
    /// as a whole. Check the resulting [`Binder`][] for errors relating to individual
    /// modules that could not be loaded.
    ///
    /// **Actually, that's just the plan... Right now, the result is actually the whole
    /// result.** We'll get around to error handling eventually.
    pub fn load(self) -> crate::Result<Binder<Parse<Document>>> {
        let absolute_path = std::env::current_dir()
            .map_err(|error| Error::new(ErrorKind::Inaccessible, error))?
            .join(&self.base_path);
        let location = Location::local_absolute(absolute_path);
        let binder = WipBinder::new(&self.cache);
        binder.load(location)
    }
}
