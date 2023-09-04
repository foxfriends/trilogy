use crate::location::Location;
use trilogy_parser::syntax::{DefinitionItem, Document, ModuleDefinition};
use trilogy_parser::{Parse, Parser};
use trilogy_scanner::Scanner;

#[derive(Clone, Debug)]
pub struct Module<T> {
    pub(super) location: Location,
    pub(super) contents: T,
}

impl Module<Parse<Document>> {
    pub(super) fn new(location: Location, source: &str) -> Self {
        let scanner = Scanner::new(source);
        let parser = Parser::new(scanner);
        let contents = parser.parse();
        Self { location, contents }
    }

    pub(super) fn imported_modules(&self) -> impl Iterator<Item = Location> + '_ {
        fn module_imported_modules(module_def: &ModuleDefinition) -> Vec<&str> {
            module_def
                .definitions
                .iter()
                .flat_map(|def| match &def.item {
                    DefinitionItem::Module(module_def) => module_imported_modules(module_def),
                    DefinitionItem::ExternalModule(module_def) => vec![module_def.locator.as_ref()],
                    _ => vec![],
                })
                .collect()
        }

        self.contents
            .ast()
            .definitions
            .iter()
            .flat_map(|def| match &def.item {
                DefinitionItem::Module(module_def) => module_imported_modules(module_def),
                DefinitionItem::ExternalModule(module_def) => vec![module_def.locator.as_ref()],
                _ => vec![],
            })
            .map(|import| self.location.relative(import))
    }
}
