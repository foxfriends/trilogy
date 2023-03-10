use super::*;
use crate::{Analyzer, LexicalError};
use source_span::Span;
use std::collections::HashMap;
use trilogy_parser::syntax::{DefinitionItem, Document};
use trilogy_parser::Spanned;

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub renames: Vec<Rename>,
    pub items: HashMap<ItemKey, Vec<Item>>,
    pub tests: Vec<Test>,
    pub exported_items: HashMap<String, Export>,
}

impl Module {
    pub(crate) fn analyze(analyzer: &mut Analyzer, ast: Document) -> Self {
        let span = ast.span();
        let renames = vec![];
        let items = HashMap::new();
        let tests = vec![];
        let mut exported_items: HashMap<String, Export> = HashMap::new();

        for def in ast.definitions {
            match def.item {
                DefinitionItem::Export(export_definition) => {
                    for identifier in export_definition.names {
                        let export = Export {
                            span: identifier.span(),
                            name: identifier.into(),
                        };
                        if exported_items.contains_key(&export.name) {
                            analyzer.error(LexicalError::ExportedMultipleTimes {
                                original: exported_items.get(&export.name).unwrap().span,
                                duplicate: export.span,
                                name: export.name,
                            });
                            continue;
                        }
                        exported_items.insert(export.name.clone(), export);
                    }
                }
                DefinitionItem::Import(_import) => {}
                DefinitionItem::ModuleImport(_module) => {}
                DefinitionItem::Module(_module) => {}
                DefinitionItem::ExternalModule(_module) => {}
                DefinitionItem::Function(_func) => {}
                DefinitionItem::Procedure(_proc) => {}
                DefinitionItem::Rule(_rule) => {}
                DefinitionItem::Test(_test) => {}
            }
        }

        Self {
            span,
            renames,
            items,
            tests,
            exported_items,
        }
    }
}
