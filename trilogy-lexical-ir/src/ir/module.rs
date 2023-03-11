use super::*;
use crate::{Analyzer, LexicalError};
use source_span::Span;
use std::collections::{HashMap, HashSet};
use trilogy_parser::syntax::{Alias, DefinitionItem, Document};
use trilogy_parser::Spanned;

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub imported_modules: HashMap<Id, Evaluation>,
    pub imported_items: HashMap<Id, Evaluation>,
    pub items: HashMap<ItemKey, Vec<Item>>,
    pub tests: Vec<Test>,
    pub exported_items: HashMap<String, Export>,
}

impl Module {
    pub(crate) fn analyze(analyzer: &mut Analyzer, ast: Document) -> Self {
        let span = ast.span();
        let imported_modules = HashMap::new();
        let imported_items = HashMap::new();
        let items = HashMap::new();
        let tests = vec![];
        let mut exported_items: HashMap<String, Export> = HashMap::new();

        let _top_level_ids = Self::extract_top_level_ids(analyzer, &ast);

        for def in ast.definitions {
            match def.item {
                DefinitionItem::Export(export_definition) => {
                    for identifier in export_definition.names {
                        let export = Export {
                            span: identifier.span(),
                            name: identifier.as_ref().to_owned(),
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
            imported_modules,
            imported_items,
            items,
            tests,
            exported_items,
        }
    }

    fn extract_top_level_ids(analyzer: &mut Analyzer, ast: &Document) -> HashSet<Id> {
        let mut top_level_ids: HashSet<Id> = HashSet::default();
        let mut top_level_names: HashMap<String, (Id, Span)> = HashMap::default();
        let mut add_name = |id: Id, span: Span| match top_level_names.get(id.name().unwrap()) {
            Some((existing, original)) if id != *existing => {
                analyzer.error(LexicalError::ConflictingDefinition {
                    name: id.name().unwrap().to_owned(),
                    original: *original,
                    conflict: span,
                });
            }
            _ => {
                top_level_names.insert(id.name().unwrap().to_owned(), (id.clone(), span));
                top_level_ids.insert(id);
            }
        };

        for def in &ast.definitions {
            match &def.item {
                DefinitionItem::Export(..) | DefinitionItem::Test(..) => continue,
                DefinitionItem::Import(import) => {
                    import
                        .names
                        .iter()
                        .map(|alias| match &alias {
                            Alias::Same(name) => name,
                            Alias::Rename(.., name) => name,
                        })
                        .for_each(|name| add_name(Id::new_immutable(name), name.span()));
                }
                DefinitionItem::ModuleImport(module) => {
                    add_name(
                        Id::new_item(ItemKey::new_module(&module.name, 0)),
                        module.name.span(),
                    );
                }
                DefinitionItem::Module(module) => add_name(
                    Id::new_item(ItemKey::new_module(
                        &module.head.name,
                        module.head.parameters.len(),
                    )),
                    module.head.span(),
                ),
                DefinitionItem::ExternalModule(module) => add_name(
                    Id::new_item(ItemKey::new_module(&module.head.name, 0)),
                    module.head.span(),
                ),
                DefinitionItem::Function(func) => add_name(
                    Id::new_item(ItemKey::new_func(
                        &func.head.name,
                        func.head.parameters.len(),
                    )),
                    func.head.span(),
                ),
                DefinitionItem::Procedure(proc) => add_name(
                    Id::new_item(ItemKey::new_func(
                        &proc.head.name,
                        proc.head.parameters.len(),
                    )),
                    proc.head.span(),
                ),
                DefinitionItem::Rule(rule) => add_name(
                    Id::new_item(ItemKey::new_func(
                        &rule.head.name,
                        rule.head.parameters.len(),
                    )),
                    rule.head.span(),
                ),
            }
        }

        top_level_ids
    }
}
