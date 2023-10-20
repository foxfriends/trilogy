use crate::LoadError;
use crate::{location::Location, trilogy::load_error::ErrorKind};
use std::collections::HashMap;
use trilogy_ir::{ir, Analyzer, Resolver};
use trilogy_parser::syntax::Document;

impl Resolver for Location {
    fn resolve(&self, locator: &str) -> String {
        self.relative(locator).to_string()
    }
}

pub fn analyze<E: std::error::Error>(
    documents: Vec<(Location, Document)>,
) -> Result<HashMap<Location, ir::Module>, LoadError<E>> {
    let mut analyzed = HashMap::default();
    let mut errors = LoadError::new_empty();

    for (location, document) in documents {
        let mut analyzer = Analyzer::new(&location);
        let module = analyzer.analyze(document);
        for error in analyzer.errors() {
            errors.add(ErrorKind::Analyzer(location.clone(), error));
        }
        analyzed.insert(location, module);
    }

    if errors.is_empty() {
        Ok(analyzed)
    } else {
        Err(errors)
    }
}
