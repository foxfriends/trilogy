#[cfg(feature = "std")]
use crate::stdlib;

use super::{Source, Trilogy};
use crate::location::Location;
#[cfg(feature = "std")]
use crate::FileSystemCache;
use crate::{Cache, NoopCache};
#[cfg(feature = "std")]
use home::home_dir;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use trilogy_vm::Native;

mod analyzer;
mod converter;
mod error;
mod loader;
mod report;

pub use error::Error;
pub use report::Report;
use report::ReportBuilder;

/// Builder for instances of [`Trilogy`][].
///
/// If looking to supply your own native modules to a Trilogy program you have written,
/// you will be using this Builder to provide those.
pub struct Builder<C: Cache + 'static> {
    root_dir: Option<PathBuf>,
    asm_modules: HashMap<Location, String>,
    native_modules: HashMap<Location, Native>,
    source_modules: HashMap<Location, String>,
    is_library: bool,
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
            .map(stdlib::apply)
    }

    fn map(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
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
            asm_modules: HashMap::new(),
            native_modules: HashMap::new(),
            source_modules: HashMap::new(),
            is_library: false,
            cache: NoopCache,
        }
    }
}

impl<C: Cache> Builder<C> {
    /// Adds a native module to this builder as a library.
    ///
    /// The location describes how Trilogy code should reference this module.
    pub fn native_module<N: Into<Native>>(mut self, location: Location, library: N) -> Self {
        self.native_modules.insert(location, library.into());
        self
    }

    /// Adds an ASM module to this builder as a library.
    ///
    /// This module is parsed as ASM directly, and must correctly implement the Trilogy interfaces,
    /// otherwise it is undefined behaviour as to what happens when it is executed.
    pub fn asm_module(mut self, location: Location, source: String) -> Self {
        self.asm_modules.insert(location, source);
        self
    }

    /// Adds a Trilogy source module to this builder as a library.
    ///
    /// Any other dependencies of this native module must also be added manually, otherwise
    /// module resolution will later fail.
    pub fn source_module(mut self, location: Location, source: String) -> Self {
        self.source_modules.insert(location, source);
        self
    }

    /// Sets the module cache for this Builder. The module cache is used when building
    /// the Trilogy instance to load modules previously loaded from the Internet from
    /// somewhere hopefully faster to reach.
    pub fn with_cache<C2: Cache>(self, cache: C2) -> Builder<C2> {
        Builder {
            root_dir: self.root_dir,
            asm_modules: self.asm_modules,
            native_modules: self.native_modules,
            source_modules: self.source_modules,
            is_library: false,
            cache,
        }
    }

    /// Sets this builder to being in library mode, where having a `proc main!()` in the
    /// entrypoint is __not__ required.
    ///
    /// Note that this means the resulting Trilogy instance may not be able to be [`run`][Trilogy::run] as a
    /// program directly, if there really is no `proc main!()` exported. Specific exported
    /// functions may be called directly using [`call`][`Trilogy::call`].
    pub fn is_library(mut self, is_library: bool) -> Self {
        self.is_library = is_library;
        self
    }

    /// Build a Trilogy instance from a Trilogy source file.
    ///
    /// # Errors
    ///
    /// Returns an error report when there are any errors in the Trilogy source
    /// file, such as syntax errors or other structural errors.
    ///
    /// Note that while a successful result does indicate that the source contained a
    /// valid piece of Trilogy code, it is not necessarily a valid program that can be
    /// run. In particular, libraries are valid code but cannot be run.
    pub fn build_from_source(self, file: impl AsRef<Path>) -> Result<Trilogy, Report<C::Error>> {
        log::trace!("begin constructing Trilogy program");
        let Self {
            mut cache,
            root_dir,
            asm_modules,
            native_modules,
            source_modules,
            is_library,
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
        let time_loading = Instant::now();
        let documents = loader::load(&cache, &entrypoint, &source_modules, &mut report);
        cache = report.checkpoint(&root_path, cache)?;
        log::trace!("all modules loaded: {:?}", time_loading.elapsed());

        let time_analyzing = Instant::now();
        let mut modules = converter::convert(documents, &mut report);
        analyzer::analyze(&mut modules, &entrypoint, &mut report, is_library);
        report.checkpoint(&root_path, cache)?;
        log::trace!("program analyzed: {:?}", time_analyzing.elapsed());

        Ok(Trilogy::new(
            Source::Trilogy {
                modules,
                asm_modules,
                entrypoint,
            },
            native_modules,
        ))
    }

    /// Build a Trilogy instance from a pre-compiled Trilogy ASM file.
    ///
    /// # Errors
    ///
    /// This method will only error if the ASM file cannot be read. Actual errors in the
    /// file's contents will only be detected when it comes time to run the contained program.
    pub fn build_from_asm(self, file: &mut dyn Read) -> Result<Trilogy, std::io::Error> {
        let mut asm = String::new();
        file.read_to_string(&mut asm)?;
        Ok(Trilogy::new(Source::Asm { asm }, self.native_modules))
    }
}
