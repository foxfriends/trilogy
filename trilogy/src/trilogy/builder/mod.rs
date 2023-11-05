use crate::location::Location;
use crate::{Cache, FileSystemCache, NativeModule, NoopCache};
use home::home_dir;

#[cfg(feature = "std")]
use crate::stdlib;

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use self::report::ReportBuilder;

use super::{Source, Trilogy};

mod analyzer;
mod converter;
mod error;
mod loader;
mod report;

pub use error::Error;
pub use report::Report;

/// Builder for instances of [`Trilogy`][].
///
/// If looking to supply your own native modules to a Trilogy program you have written,
/// you will be using this Builder to provide those.
pub(crate) struct Builder<C: Cache + 'static> {
    root_dir: Option<PathBuf>,
    libraries: HashMap<Location, NativeModule>,
    cache: C,
}

#[cfg(feature = "std")]
impl Builder<FileSystemCache> {
    /// Creates a new Trilogy builder that is configured as "standard".
    ///
    /// Programs created from this builder use the default resolver and come
    /// loaded with the standard library (imported as `trilogy:std`).
    ///
    /// The default resolver expects the existence of a file system with a home directory and uses
    /// the directory `$HOME/.trilogy/cache` to cache Trilogy modules downloaded from the Internet.
    pub fn std() -> Self {
        let home = home_dir()
            .expect("home dir should exist")
            .join(".trilogy/cache");
        Builder::new()
            .with_cache(
                FileSystemCache::new(home)
                    .expect("canonical cache dir ~/.trilogy/cache is occupied"),
            )
            .library(Location::library("io").unwrap(), stdlib::io())
            .library(Location::library("str").unwrap(), stdlib::str())
            .library(Location::library("num").unwrap(), stdlib::num())
    }
}

impl Default for Builder<NoopCache> {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder<NoopCache> {
    /// Creates a new Trilogy builder with nothing added.
    ///
    /// Programs created from this builder will not have the standard library (unless you manually
    /// re-add it).
    ///
    /// This builder also does not come with a cache for some reason.
    pub fn new() -> Self {
        Self {
            root_dir: None,
            libraries: HashMap::new(),
            cache: NoopCache,
        }
    }
}

impl<C: Cache> Builder<C> {
    /// Adds a native module to this builder as a library.
    ///
    /// The location describes how Trilogy code should reference this library.
    pub fn library(mut self, location: Location, library: NativeModule) -> Self {
        self.libraries.insert(location, library);
        self
    }

    /// Sets the module cache for this Builder. The module cache is used when building
    /// the Trilogy instance to load modules previously loaded from the Internet from
    /// somewhere hopefully faster to reach.
    pub fn with_cache<C2: Cache>(self, cache: C2) -> Builder<C2> {
        Builder {
            root_dir: self.root_dir,
            libraries: self.libraries,
            cache,
        }
    }

    pub(super) fn build_from_source(
        self,
        file: impl AsRef<Path>,
    ) -> Result<Trilogy, Report<C::Error>> {
        let Self {
            mut cache,
            root_dir,
            libraries,
        } = self;
        let mut report = ReportBuilder::default();
        let root_path = match root_dir {
            Some(root_dir) => root_dir,
            None => match std::env::current_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    report.error(Error::external(error));
                    return Err(report.report(file.as_ref().to_owned(), cache));
                }
            },
        };
        let entrypoint = Location::entrypoint(root_path.clone(), file);
        let documents = loader::load(&cache, &entrypoint, &mut report);
        cache = report.checkpoint(&root_path, cache)?;
        let mut modules = converter::convert(documents, &mut report);
        analyzer::analyze(&mut modules, &entrypoint, &mut report);
        report.checkpoint(&root_path, cache)?;
        Ok(Trilogy::new(
            Source::Trilogy {
                modules,
                entrypoint,
            },
            libraries,
        ))
    }

    pub(super) fn build_from_asm(self, file: &mut dyn Read) -> Result<Trilogy, std::io::Error> {
        let mut asm = String::new();
        file.read_to_string(&mut asm)?;
        Ok(Trilogy::new(Source::Asm { asm }, self.libraries))
    }
}
