use crate::loader::WipBinder;
use crate::location::Location;
use crate::{Cache, LoadError, NativeModule, NoopCache};

#[cfg(feature = "std")]
use crate::stdlib;

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::Path;

use super::Trilogy;

pub struct Builder<E> {
    libraries: HashMap<&'static str, NativeModule>,
    cache: Box<dyn Cache<Error = E>>,
}

impl Default for Builder<Infallible> {
    fn default() -> Self {
        let mut builder = Self::new();
        #[cfg(feature = "std")]
        {
            builder = builder.library("std", stdlib::std());
        }
        builder
    }
}

impl Builder<Infallible> {
    pub fn new() -> Self {
        Self {
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
            libraries: self.libraries,
            cache: Box::new(cache),
        }
    }

    pub(super) fn build_from_file(self, file: impl AsRef<Path>) -> Result<Trilogy, LoadError<E>> {
        let absolute_path = std::env::current_dir()
            .map_err(|error| LoadError::External(Box::new(error)))?
            .join(file);
        let location = Location::local_absolute(absolute_path);
        let wip_binder = WipBinder::new(&*self.cache);
        let binder = wip_binder.load(location)?;
        if binder.has_errors() {
            return Err(LoadError::Syntax(binder.errors().cloned().collect()));
        }
        let program = match binder.analyze(&self.libraries) {
            Ok(program) => program,
            Err(errors) => return Err(LoadError::Linker(errors)),
        };
        let program = program.generate_code();
        Ok(Trilogy::from(program))
    }
}
