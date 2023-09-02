use crate::NativeModule;

use super::linker::{Linker, LinkerError};
use super::{Location, Module, Program};
use std::collections::HashMap;
use trilogy_parser::syntax::{Document, SyntaxError};
use trilogy_parser::Parse;
use url::Url;

#[derive(Clone, Debug)]
pub struct Binder<T> {
    entrypoint: Location,
    pub modules: HashMap<Url, Module<T>>,
}

impl<T> Binder<T> {
    pub(super) fn new(entrypoint: Location) -> Self {
        Self {
            entrypoint,
            modules: HashMap::default(),
        }
    }
}

impl Binder<Parse<Document>> {
    pub fn has_errors(&self) -> bool {
        self.modules
            .values()
            .any(|module| module.contents.has_errors())
    }

    pub fn errors(&self) -> impl Iterator<Item = &SyntaxError> {
        self.modules
            .values()
            .flat_map(|module| module.contents.errors())
    }

    pub fn analyze(
        self,
        _libraries: &HashMap<&'static str, NativeModule>,
    ) -> Result<Program, Vec<LinkerError>> {
        let mut linker = Linker::new(self.modules);
        linker.link_module(&self.entrypoint);
        if linker.has_errors() {
            Err(linker.into_errors())
        } else {
            Ok(Program::new(linker.into_module(&self.entrypoint)))
        }
    }
}
