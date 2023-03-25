use super::*;
use crate::{Analyzer, Error};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub(super) enum DefinitionItem {
    Procedure(Box<ProcedureDefinition>),
    Function(Box<FunctionDefinition>),
    Rule(Box<RuleDefinition>),
    Test(Box<TestDefinition>),
    Alias(Box<Alias>),
    Module(Box<ModuleDefinition>),
}

#[derive(Clone, Debug)]
pub struct Definition {
    span: Span,
    pub(super) item: DefinitionItem,
    pub is_exported: bool,
}

impl Definition {
    pub(super) fn convert_into(
        analyzer: &mut Analyzer,
        ast: syntax::Definition,
        definitions: &mut Definitions,
    ) {
        match ast.item {
            syntax::DefinitionItem::Export(ast) => {
                for name in ast.names {
                    match analyzer.declared(name.as_ref()) {
                        Some(id) => {
                            definitions.get_mut(id).unwrap().is_exported = true;
                        }
                        None => {
                            analyzer.error(Error::UnknownExport { name: name.clone() });
                        }
                    }
                }
            }
            syntax::DefinitionItem::ExternalModule(..) => todo!(),
            syntax::DefinitionItem::Function(..) => todo!(),
            syntax::DefinitionItem::Import(..) => todo!(),
            syntax::DefinitionItem::Module(..) => todo!(),
            syntax::DefinitionItem::ModuleImport(..) => todo!(),
            syntax::DefinitionItem::Procedure(..) => todo!(),
            syntax::DefinitionItem::Rule(..) => todo!(),
            syntax::DefinitionItem::Test(ast) => {
                definitions.push(Definition {
                    span: ast.span(),
                    item: DefinitionItem::Test(Box::new(TestDefinition::convert(analyzer, *ast))),
                    is_exported: false,
                });
            }
        }
    }

    pub(super) fn declare(analyzer: &mut Analyzer, ast: &syntax::Definition) -> Option<Self> {
        let def = match &ast.item {
            syntax::DefinitionItem::Export(..) => return None,
            syntax::DefinitionItem::ExternalModule(ast) => {
                Identifier::declare(analyzer, ast.head.name.clone());
                return None;
            }
            syntax::DefinitionItem::Function(ast) => {
                let span = ast.span();
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span,
                    item: DefinitionItem::Function(Box::new(FunctionDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Import(ast) => {
                for alias in &ast.names {
                    match alias {
                        syntax::Alias::Same(name) => {
                            Identifier::declare(analyzer, name.clone());
                        }
                        syntax::Alias::Rename(_, name) => {
                            Identifier::declare(analyzer, name.clone());
                        }
                    }
                }
                return None;
            }
            syntax::DefinitionItem::Module(ast) => {
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span: ast.span(),
                    item: DefinitionItem::Module(Box::new(ModuleDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::ModuleImport(ast) => {
                Identifier::declare(analyzer, ast.name.clone());
                return None;
            }
            syntax::DefinitionItem::Procedure(ast) => {
                let span = ast.span();
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span,
                    item: DefinitionItem::Procedure(Box::new(ProcedureDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Rule(ast) => {
                let span = ast.span();
                let name = Identifier::declare(analyzer, ast.head.name.clone());
                Self {
                    span,
                    item: DefinitionItem::Rule(Box::new(RuleDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Test(..) => return None,
        };

        Some(def)
    }
}
