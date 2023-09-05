use crate::{loader::Program, location::Location};
use crate::{LoadError, NativeModule};
use std::collections::HashMap;
use std::sync::Arc;
use trilogy_ir::{ir, Analyzer, Resolver};
use trilogy_parser::syntax::Document;

pub(super) struct Linker<'a> {
    #[allow(dead_code)]
    libraries: &'a HashMap<&'static str, NativeModule>,
    unlinked: HashMap<Location, Document>,
    linked: HashMap<Location, Arc<ir::ModuleCell>>,
    errors: Vec<trilogy_ir::Error>,
}

impl<'a> Linker<'a> {
    pub(super) fn new(
        libraries: &'a HashMap<&'static str, NativeModule>,
        unlinked: HashMap<Location, Document>,
    ) -> Self {
        Self {
            libraries,
            unlinked,
            linked: HashMap::new(),
            errors: vec![],
        }
    }

    pub(super) fn link_module(&mut self, location: &Location) {
        if self.linked.contains_key(location) {
            return;
        }

        let module_cell = Arc::<ir::ModuleCell>::default();
        self.linked.insert(location.clone(), module_cell.clone());

        let module = self
            .unlinked
            .remove(location.as_ref())
            .expect("all modules should have been successfully located already");
        let mut analyzer = Analyzer::new(LinkerResolver {
            location,
            linker: self,
        });
        let module = analyzer.analyze(module);
        let errors = analyzer.errors();
        self.errors.extend(errors);
        module_cell.insert(module);
    }

    pub(super) fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub(super) fn into_errors(self) -> Vec<trilogy_ir::Error> {
        self.errors
    }

    pub(super) fn into_module(mut self, location: &Location) -> Arc<ir::ModuleCell> {
        self.linked.remove(location).unwrap()
    }
}

struct LinkerResolver<'a, 'b> {
    location: &'a Location,
    linker: &'a mut Linker<'b>,
}

impl Resolver for LinkerResolver<'_, '_> {
    fn resolve(&mut self, path: &str) -> Arc<ir::ModuleCell> {
        let location = self.location.relative(path);
        self.linker.link_module(&location);
        self.linker.linked.get(&location).unwrap().clone()
    }

    fn location(&self) -> String {
        self.location.to_string()
    }
}

pub fn link<E: std::error::Error>(
    libraries: &HashMap<&'static str, NativeModule>,
    modules: HashMap<Location, Document>,
    entrypoint: Location,
) -> Result<Program, LoadError<E>> {
    let mut linker = Linker::new(libraries, modules);
    linker.link_module(&entrypoint);
    if linker.has_errors() {
        Err(LoadError::Linker(linker.into_errors()))
    } else {
        Ok(Program::new(linker.into_module(&entrypoint)))
    }
}
