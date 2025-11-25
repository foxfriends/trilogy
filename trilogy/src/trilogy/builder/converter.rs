use super::Error;
use super::loader::Module;
use super::report::ReportBuilder;
use crate::location::Location;
use std::collections::HashMap;
use trilogy_ir::{Converter, Resolver, ir};

impl Resolver for Location {
    fn resolve(&self, locator: &str) -> String {
        self.relative(locator).to_string()
    }
}

pub(super) fn convert<E: std::error::Error>(
    modules: HashMap<Location, Module>,
    report: &mut ReportBuilder<E>,
) -> HashMap<Location, ir::Module> {
    let mut converted = HashMap::default();

    for (location, module) in modules {
        let mut converter = Converter::new(&location, &module.source);
        let module = converter.convert(module.contents.into_ast());
        for error in converter.errors() {
            report.error(Error::ir(location.clone(), error));
        }
        converted.insert(location, module);
    }

    converted
}
