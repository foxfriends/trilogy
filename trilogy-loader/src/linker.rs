use crate::{Location, Module};
use reqwest::Url;
use std::collections::HashMap;
use std::sync::Arc;
use trilogy_ir::{ir, Resolver};

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

        let mut module = self
            .unlinked
            .remove(location.as_ref())
            .expect("all modules should have been successfully located already")
            .contents;

        for definition in module.definitions_mut() {
            if let Some(module_reference) = definition.as_module_mut() {
                module_reference.resolve(&mut LinkerResolver {
                    location: &location,
                    linker: self,
                });
            }
        }

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

struct LinkerResolver<'a> {
    location: &'a Location,
    linker: &'a mut Linker,
}

impl Resolver for LinkerResolver<'_> {
    fn resolve(&mut self, path: &str) -> Arc<ir::Module> {
        let location = self.location.relative(path);
        self.linker.link_module(location.clone());
        self.linker.linked.get(&location).unwrap().clone()
    }
}
