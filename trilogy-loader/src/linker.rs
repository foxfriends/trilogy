use crate::{Location, Module};
use reqwest::Url;
use std::collections::HashMap;
use std::sync::Arc;
use trilogy_ir::ir;

pub enum LinkerError {}

pub(crate) struct Linker {
    unlinked: HashMap<Url, Module<ir::Module>>,
    linked: HashMap<Location, Arc<ir::Module>>,
    errors: Vec<LinkerError>,
}

impl Linker {
    pub(crate) fn new(unlinked: HashMap<Url, Module<ir::Module>>) -> Self {
        Self {
            unlinked,
            linked: HashMap::new(),
            errors: vec![],
        }
    }

    pub(crate) fn link_module(&mut self, location: Location) {
        if self.linked.contains_key(&location) {
            return;
        }

        let module = self
            .unlinked
            .remove(location.as_ref())
            .expect("all modules should have been successfully located already")
            .contents;

        self.linked.insert(location, Arc::new(module));
    }

    pub(crate) fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub(crate) fn into_errors(self) -> Vec<LinkerError> {
        self.errors
    }

    pub(crate) fn into_module(mut self, location: Location) -> Arc<ir::Module> {
        self.linked.remove(&location).unwrap()
    }
}
