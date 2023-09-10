use crate::{loader::Program, location::Location};
use crate::{LoadError, NativeModule};
use std::collections::HashMap;
use std::sync::Arc;
use trilogy_ir::{ir, Analyzer, Resolver};
use trilogy_parser::syntax::Document;

pub fn analyze<E: std::error::Error>(
    modules: HashMap<Location, Document>,
) -> Result<Program, LoadError<E>> {
    let mut analyzed = HashMap::default();
    let mut errors = vec![];

    for (location, module) in modules {
        let analyzer = Analyzer::new();
        let module = analyzer.analyze(module);
        errors.extend(analyzer.errors());
        analyzed.insert(location, module);
    }

    if errors.is_empty() {
        Ok(Program::new(analyzer.into_module(&entrypoint)))
    } else {
        Err(LoadError::Analyzer(analyzer.into_errors()))
    }
}
