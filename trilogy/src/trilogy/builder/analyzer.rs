use super::report::ReportBuilder;
use super::Error;
use crate::location::Location;
use std::collections::HashMap;
use trilogy_ir::{ir, Analyzer, Resolver};
use trilogy_parser::syntax::Document;

impl Resolver for Location {
    fn resolve(&self, locator: &str) -> String {
        self.relative(locator).to_string()
    }
}

pub(super) fn analyze<E: std::error::Error>(
    documents: Vec<(Location, Document)>,
    report: &mut ReportBuilder<E>,
) -> HashMap<Location, ir::Module> {
    let mut analyzed = HashMap::default();

    for (location, document) in documents {
        let mut analyzer = Analyzer::new(&location);
        let module = analyzer.analyze(document);
        for error in analyzer.errors() {
            report.error(Error::semantic(location.clone(), error));
        }
        analyzed.insert(location, module);
    }

    analyzed
}
