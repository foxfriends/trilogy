use super::Error;
use super::report::ReportBuilder;
use crate::location::Location;
use std::collections::HashMap;
use trilogy_ir::{Converter, Resolver, ir};
use trilogy_parser::syntax::Document;

impl Resolver for Location {
    fn resolve(&self, locator: &str) -> String {
        self.relative(locator).to_string()
    }
}

pub(super) fn convert<E: std::error::Error>(
    documents: Vec<(Location, Document)>,
    report: &mut ReportBuilder<E>,
) -> HashMap<Location, ir::Module> {
    let mut converted = HashMap::default();

    for (location, document) in documents {
        let mut converter = Converter::new(&location);
        let module = converter.convert(document);
        for error in converter.errors() {
            report.error(Error::ir(location.clone(), error));
        }
        converted.insert(location, module);
    }

    converted
}
