use crate::{Location, Module, Program};
use std::collections::HashMap;
use trilogy_ir::{ir, Analyzer, Error};
use trilogy_parser::syntax::{Document, SyntaxError};
use trilogy_parser::Parse;
use url::Url;

#[derive(Clone, Debug)]
pub struct Binder<T> {
    entrypoint: Location,
    pub modules: HashMap<Url, Module<T>>,
}

impl<T> Binder<T> {
    pub(crate) fn new(entrypoint: Location) -> Self {
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

    pub fn analyze(self) -> Result<Binder<ir::Module>, Vec<Error>> {
        let mut errors = vec![];
        let mut updated = HashMap::new();
        for (url, module) in self.modules {
            let upgraded = module.upgrade(|contents| {
                let ast = contents.into_ast();
                let mut analyzer = Analyzer::new();
                let module = analyzer.analyze(ast);
                errors.extend(analyzer.errors());
                module
            });
            updated.insert(url, upgraded);
        }
        Ok(Binder {
            entrypoint: self.entrypoint,
            modules: updated,
        })
    }
}

impl Binder<ir::Module> {
    #[allow(clippy::result_unit_err)]
    pub fn link(self) -> Result<Program, ()> {
        let program = Program::new();
        Ok(program)
    }
}
