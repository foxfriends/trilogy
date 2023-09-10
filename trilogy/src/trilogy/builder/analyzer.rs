use crate::location::Location;
use crate::LoadError;
use std::collections::HashMap;
use trilogy_ir::{ir, Analyzer};
use trilogy_parser::syntax::Document;

pub fn analyze<E: std::error::Error>(
    modules: HashMap<Location, Document>,
) -> Result<HashMap<Location, ir::Module>, LoadError<E>> {
    let mut analyzed = HashMap::default();
    let mut errors = vec![];

    for (location, module) in modules {
        let mut analyzer = Analyzer::new();
        let module = analyzer.analyze(module);
        errors.extend(analyzer.errors());
        analyzed.insert(location, module);
    }

    if errors.is_empty() {
        Ok(analyzed)
    } else {
        Err(LoadError::Analyzer(errors))
    }
}
