use home::home_dir;

use crate::location::Location;
use crate::{Cache, FileSystemCache, LoadError, NativeModule, NoopCache};

#[cfg(feature = "std")]
use crate::stdlib;

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::{Path, PathBuf};

use super::Trilogy;

mod analyzer;
mod linker;
mod loader;

pub struct Builder<E> {
    root_dir: Option<PathBuf>,
    libraries: HashMap<Location, NativeModule>,
    cache: Box<dyn Cache<Error = E>>,
}

#[cfg(feature = "std")]
impl Builder<std::io::Error> {
    pub fn std() -> Self {
        let home = home_dir()
            .expect("home dir should exist")
            .join(".trilogy/cache");
        Builder::new()
            .with_cache(
                FileSystemCache::new(home)
                    .expect("canonical cache dir ~/.trilogy/cache is occupied"),
            )
            .library(Location::library("std").unwrap(), stdlib::std())
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
    pub fn library(mut self, location: Location, library: NativeModule) -> Self {
        self.libraries.insert(location, library);
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
        let ir = analyzer::analyze(binder)?;
        let program = linker::link(libraries, ir, entrypoint);
        let program = program.generate_code();
        Ok(Trilogy::from(program))
    }
}
