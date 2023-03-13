use super::*;
use crate::{Analyzer, LexicalError};
use source_span::Span;
use std::collections::HashMap;
use trilogy_lexical_ir::{EitherModule, Export, ExternalModule, Id, Item, ItemKey, Module, Value};
use trilogy_parser::syntax::{
    Alias, Definition, DefinitionItem, FunctionDefinition, ModuleDefinition, ProcedureDefinition,
    RuleDefinition,
};
use trilogy_parser::Spanned;

pub(crate) fn analyze_definitions(
    analyzer: &mut Analyzer,
    span: Span,
    definitions: Vec<Definition>,
) -> Module {
    let mut imported_modules = HashMap::new();
    let mut imported_items = HashMap::new();
    let mut submodules: HashMap<ItemKey, EitherModule> = HashMap::new();
    let mut items: HashMap<ItemKey, Vec<Item>> = HashMap::new();
    let mut tests = vec![];
    let mut exported_items: HashMap<String, Export> = HashMap::new();

    let top_level_ids = extract_top_level_ids(analyzer, &definitions);
    analyzer.scope_mut().extend(top_level_ids);

    for def in definitions {
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
            DefinitionItem::Import(import) => {
                let span = import.span();
                let module_ref = analyze_module_path(analyzer, import.module);
                for name in import.names {
                    let (from, to) = match name {
                        Alias::Same(ref name) => (name.clone(), name),
                        Alias::Rename(from, ref to) => (from, to),
                    };
                    let id = analyzer.scope().find(to.as_ref()).unwrap();
                    imported_items.insert(
                        id,
                        Value::access(module_ref.clone(), {
                            let span = from.span();
                            Value::static_resolve(from).at(span)
                        })
                        .at(span),
                    );
                }
            }
            DefinitionItem::ModuleImport(import) => {
                let module = analyze_module_path(analyzer, import.module);
                let id = analyzer.scope().find(import.name.as_ref()).unwrap();
                imported_modules.insert(id, module);
            }
            DefinitionItem::Module(module) => {
                let key = module_key(&module);
                if let Some(previous) = submodules.get(&key) {
                    analyzer.error(LexicalError::ConflictingDefinition {
                        name: key.name,
                        original: previous.span(),
                        conflict: module.span(),
                    });
                    continue;
                }
                let item = analyze_module(analyzer, *module);
                submodules.insert(key, EitherModule::from(item));
            }
            DefinitionItem::ExternalModule(module) => {
                let key = ItemKey::new_module(&module.head.name, 0);
                submodules.insert(
                    key,
                    EitherModule::from(ExternalModule {
                        span: module.span(),
                        locator: module.locator.into(),
                    }),
                );
            }
            DefinitionItem::Function(func) => {
                let key = func_key(&func);
                let item = analyze_func(analyzer, *func);
                items.entry(key).or_default().push(item);
            }
            DefinitionItem::Procedure(proc) => {
                let key = proc_key(&proc);
                if let Some(previous) = items.get(&key).and_then(|vec| vec.first()) {
                    analyzer.error(LexicalError::ConflictingDefinition {
                        name: key.name,
                        original: previous.span,
                        conflict: proc.span(),
                    });
                    continue;
                }
                let item = analyze_proc(analyzer, *proc);
                items.entry(key).or_default().push(item);
            }
            DefinitionItem::Rule(rule) => {
                let key = rule_key(&rule);
                let item = analyze_rule(analyzer, *rule);
                items.entry(key).or_default().push(item);
            }
            DefinitionItem::Test(test) => {
                tests.push(analyze_test(analyzer, *test));
            }
        }
    }

    Module {
        span,
        imported_modules,
        imported_items,
        submodules,
        items,
        tests,
        exported_items,
    }
}

fn rule_key(rule: &RuleDefinition) -> ItemKey {
    ItemKey::new_rule(&rule.head.name, rule.head.parameters.len())
}

fn proc_key(proc: &ProcedureDefinition) -> ItemKey {
    ItemKey::new_proc(&proc.head.name, proc.head.parameters.len())
}

fn func_key(func: &FunctionDefinition) -> ItemKey {
    ItemKey::new_func(&func.head.name, func.head.parameters.len())
}

fn module_key(module: &ModuleDefinition) -> ItemKey {
    ItemKey::new_module(&module.head.name, module.head.parameters.len())
}

fn extract_top_level_ids(
    analyzer: &mut Analyzer,
    definitions: &[Definition],
) -> HashMap<String, Id> {
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
        }
    };

    for def in definitions {
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
            DefinitionItem::Module(module) => {
                add_name(Id::new_item(module_key(module)), module.head.span())
            }
            DefinitionItem::ExternalModule(module) => add_name(
                Id::new_item(ItemKey::new_module(&module.head.name, 0)),
                module.head.span(),
            ),
            DefinitionItem::Function(func) => {
                add_name(Id::new_item(func_key(func)), func.head.span())
            }
            DefinitionItem::Procedure(proc) => {
                add_name(Id::new_item(proc_key(proc)), proc.head.span())
            }
            DefinitionItem::Rule(rule) => add_name(Id::new_item(rule_key(rule)), rule.head.span()),
        }
    }

    top_level_names
        .into_iter()
        .map(|(k, (v, _))| (k, v))
        .collect()
}
