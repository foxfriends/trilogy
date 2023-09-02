use super::{Location, Module};
use reqwest::Url;
use std::collections::HashMap;
use std::sync::Arc;
use trilogy_ir::{ir, Analyzer, Error as IrError, Resolver};
use trilogy_parser::{syntax::Document, Parse};

#[derive(Debug)]
pub enum LinkerError {
    Ir(IrError),
}

pub(super) struct Linker {
    unlinked: HashMap<Url, Module<Parse<Document>>>,
    linked: HashMap<Location, Arc<ir::ModuleCell>>,
    errors: Vec<LinkerError>,
}

impl Linker {
    pub(super) fn new(unlinked: HashMap<Url, Module<Parse<Document>>>) -> Self {
        Self {
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
            .expect("all modules should have been successfully located already")
            .contents;
        let mut analyzer = Analyzer::new(LinkerResolver {
            location,
            linker: self,
        });
        let module = analyzer.analyze(module.into_ast());
        let errors = analyzer.errors().into_iter().map(LinkerError::Ir);
        self.errors.extend(errors);
        module_cell.insert(module);
    }

    pub(super) fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub(super) fn into_errors(self) -> Vec<LinkerError> {
        self.errors
    }

    pub(super) fn into_module(mut self, location: &Location) -> Arc<ir::ModuleCell> {
        self.linked.remove(location).unwrap()
    }
}

struct LinkerResolver<'a> {
    location: &'a Location,
    linker: &'a mut Linker,
}

impl Resolver for LinkerResolver<'_> {
    fn resolve(&mut self, path: &str) -> Arc<ir::ModuleCell> {
        let location = self.location.relative(path);
        self.linker.link_module(&location);
        self.linker.linked.get(&location).unwrap().clone()
    }

    fn location(&self) -> String {
        self.location.to_string()
    }
}
