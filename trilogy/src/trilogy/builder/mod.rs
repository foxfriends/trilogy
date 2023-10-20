use home::home_dir;

use crate::location::Location;
use crate::{Cache, FileSystemCache, LoadError, NativeModule, NoopCache};

#[cfg(feature = "std")]
use crate::stdlib;

use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::{Source, Trilogy};

mod analyzer;
mod loader;

pub(crate) use loader::ResolverError;

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

    pub(super) fn build_from_source(self, file: impl AsRef<Path>) -> Result<Trilogy, LoadError<E>> {
        let entrypoint = Location::entrypoint(
            self.root_dir
                .map(Ok)
                .unwrap_or_else(std::env::current_dir)
                .map_err(|error| LoadError::new(vec![ResolverError::external(error).into()]))?,
            file,
        );
        let documents = loader::load(&*self.cache, &entrypoint)?;
        let modules = analyzer::analyze(documents)?;
        Ok(Trilogy::new(
            Source::Trilogy {
                modules,
                entrypoint,
            },
            self.libraries,
        ))
    }

    pub(super) fn build_from_asm(self, file: &mut dyn Read) -> Result<Trilogy, std::io::Error> {
        let mut asm = String::new();
        file.read_to_string(&mut asm)?;
        Ok(Trilogy::new(Source::Asm { asm }, self.libraries))
    }
}
