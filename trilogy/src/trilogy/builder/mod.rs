use home::home_dir;

use crate::location::Location;
use crate::{Cache, FileSystemCache, NativeModule, NoopCache};

#[cfg(feature = "std")]
use crate::stdlib;

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use self::report::ReportBuilder;

use super::{Source, Trilogy};

mod analyzer;
mod error;
mod loader;
mod report;

pub use error::Error;
pub use report::Report;

pub struct Builder<C: Cache + 'static> {
    root_dir: Option<PathBuf>,
    libraries: HashMap<Location, NativeModule>,
    cache: C,
}

#[cfg(feature = "std")]
impl Builder<FileSystemCache> {
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

impl Default for Builder<NoopCache> {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder<NoopCache> {
    pub fn new() -> Self {
        Self {
            root_dir: None,
            libraries: HashMap::new(),
            cache: NoopCache,
        }
    }
}

impl<C: Cache> Builder<C> {
    pub fn library(mut self, location: Location, library: NativeModule) -> Self {
        self.libraries.insert(location, library);
        self
    }

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
        let entrypoint = match root_dir {
            Some(root_dir) => root_dir,
            None => match std::env::current_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    report.error(Error::external(error));
                    return Err(report.report(cache));
                }
            },
        };
        let entrypoint = Location::entrypoint(entrypoint, file);
        let documents = loader::load(&cache, &entrypoint, &mut report);
        cache = report.checkpoint(cache)?;
        let modules = analyzer::analyze(documents, &mut report);
        report.checkpoint(cache)?;
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
