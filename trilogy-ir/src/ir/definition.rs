use std::sync::Arc;

use super::*;
use crate::{symbol::Symbol, Analyzer, Error, Id};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub enum DefinitionItem {
    Procedure(Box<ProcedureDefinition>),
    Function(Box<FunctionDefinition>),
    Rule(Box<RuleDefinition>),
    Test(Box<TestDefinition>),
    Module(Box<ModuleDefinition>),
}

#[derive(Clone, Debug)]
pub struct Definition {
    pub span: Span,
    pub item: DefinitionItem,
    pub is_exported: bool,
}

impl Definition {
    pub fn name(&self) -> Option<&Id> {
        match &self.item {
            DefinitionItem::Procedure(def) => Some(&def.name.id),
            DefinitionItem::Function(def) => Some(&def.name.id),
            DefinitionItem::Rule(def) => Some(&def.name.id),
            DefinitionItem::Test(..) => None,
            DefinitionItem::Module(def) => Some(&def.name.id),
        }
    }

    pub fn as_module_mut(&mut self) -> Option<&mut ModuleDefinition> {
        match &mut self.item {
            DefinitionItem::Module(module) => Some(&mut *module),
            _ => None,
        }
    }

    pub(super) fn convert_into(
        analyzer: &mut Analyzer,
        ast: syntax::Definition,
        definitions: &mut Definitions,
    ) {
        match ast.item {
            syntax::DefinitionItem::Export(ast) => {
                for name in ast.names {
                    match analyzer.declared(name.as_ref()) {
                        Some(Symbol { id, .. }) => {
                            definitions.get_mut(id).unwrap().is_exported = true;
                        }
                        None => {
                            analyzer.error(Error::UnknownExport { name: name.clone() });
                        }
                    }
                }
            }
            syntax::DefinitionItem::ExternalModule(..) => {}
            syntax::DefinitionItem::Function(ast) => {
                let Symbol { id, .. } = analyzer.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(id).unwrap();
                let DefinitionItem::Function(function) = &mut definition.item else {
                    unreachable!()
                };
                function.overloads.push(Function::convert(analyzer, *ast))
            }
            syntax::DefinitionItem::Module(ast) => {
                let Symbol { id, .. } = analyzer.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(id).unwrap();
                let DefinitionItem::Module(module) = &mut definition.item else {
                    unreachable!()
                };
                module.module = Arc::new(ModuleCell::new(Module::convert_module(analyzer, *ast)));
            }
            syntax::DefinitionItem::Procedure(ast) => {
                let Symbol { id, .. } = analyzer.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(id).unwrap();
                let DefinitionItem::Procedure(procedure) = &mut definition.item else {
                    unreachable!()
                };
                procedure.overloads.push(Procedure::convert(analyzer, *ast))
            }
            syntax::DefinitionItem::Rule(ast) => {
                let Symbol { id, .. } = analyzer.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(id).unwrap();
                let DefinitionItem::Rule(rule) = &mut definition.item else {
                    unreachable!()
                };
                rule.overloads.push(Rule::convert(analyzer, *ast))
            }
            syntax::DefinitionItem::Test(ast) => {
                definitions.push(Definition {
                    span: ast.span(),
                    item: DefinitionItem::Test(Box::new(TestDefinition::convert(analyzer, *ast))),
                    is_exported: false,
                });
            }
        }
    }

    pub(super) fn declare(analyzer: &mut Analyzer, ast: &syntax::Definition) -> Vec<Self> {
        let def = match &ast.item {
            syntax::DefinitionItem::Export(..) => return vec![],
            syntax::DefinitionItem::ExternalModule(ast) => {
                if analyzer.declared(ast.head.name.as_ref()).is_some() {
                    analyzer.error(Error::DuplicateDefinition {
                        name: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span: ast.span(),
                    item: DefinitionItem::Module(Box::new(ModuleDefinition::external(
                        name,
                        analyzer.resolve(&ast.locator.value()),
                    ))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Function(ast) => {
                if analyzer.declared(ast.head.name.as_ref()).is_some() {
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span,
                    item: DefinitionItem::Function(Box::new(FunctionDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Module(ast) => {
                if analyzer.declared(ast.head.name.as_ref()).is_some() {
                    analyzer.error(Error::DuplicateDefinition {
                        name: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span: ast.span(),
                    item: DefinitionItem::Module(Box::new(ModuleDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Procedure(ast) => {
                if analyzer.declared(ast.head.name.as_ref()).is_some() {
                    analyzer.error(Error::DuplicateDefinition {
                        name: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span,
                    item: DefinitionItem::Procedure(Box::new(ProcedureDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Rule(ast) => {
                if analyzer.declared(ast.head.name.as_ref()).is_some() {
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span,
                    item: DefinitionItem::Rule(Box::new(RuleDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Test(..) => return vec![],
        };

        vec![def]
    }
}
