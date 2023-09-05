use crate::location::Location;
use crate::{Cache, FileSystemCache, LoadError, NativeModule, NoopCache};

#[cfg(feature = "std")]
use crate::stdlib;

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};

use super::Trilogy;

mod linker;
mod loader;

pub struct Builder<E> {
    root_dir: Option<PathBuf>,
    libraries: HashMap<&'static str, NativeModule>,
    cache: Box<dyn Cache<Error = E>>,
}

#[cfg(feature = "std")]
impl Builder<std::io::Error> {
    pub fn std() -> Self {
        Builder::new()
            .with_cache(
                FileSystemCache::new("~/.trilogy")
                    .expect("canonical cache dir ~/.trilogy is occupied"),
            )
            .library("std", stdlib::std())
    }
}

impl Default for Builder<Infallible> {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder<Infallible> {
    pub fn new() -> Self {
        Self {
            root_dir: None,
            libraries: HashMap::new(),
            cache: Box::new(NoopCache),
        }
    }

    pub fn at_root<P: AsRef<Path>>(root_dir: P) -> Self {
        Self {
            root_dir: Some(root_dir.as_ref().to_owned()),
            libraries: HashMap::new(),
            cache: Box::new(NoopCache),
        }
    }
}

impl<E: std::error::Error + 'static> Builder<E> {
    pub fn library(mut self, name: &'static str, library: NativeModule) -> Self {
        self.libraries.insert(name, library);
        self
    }

    pub fn with_cache<C: Cache + 'static>(self, cache: C) -> Builder<C::Error> {
        Builder {
            root_dir: self.root_dir,
            libraries: self.libraries,
            cache: Box::new(cache),
        }
    }

    pub(super) fn build_from_file(self, file: impl AsRef<Path>) -> Result<Trilogy, LoadError<E>> {
        let entrypoint = Location::entrypoint(
            self.root_dir
                .map(Ok)
                .unwrap_or_else(std::env::current_dir)
                .map_err(|error| LoadError::External(Box::new(error)))?,
            file,
        );
        let binder = loader::load(&*self.cache, &entrypoint)?;
        let program = linker::link(&self.libraries, binder, entrypoint)?;
        let program = program.generate_code();
        Ok(Trilogy::from(program))
    }
}
