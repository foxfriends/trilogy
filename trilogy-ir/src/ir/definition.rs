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
    pub span: Span,
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
            syntax::DefinitionItem::Import(ast) => {
                let from_span = ast.from_token().span;
                let expression = Expression::convert_module_path(analyzer, ast.module);
                for alias in &ast.names {
                    let (from, to) = match alias {
                        syntax::Alias::Same(name) => (name, name),
                        syntax::Alias::Rename(from, to) => (from, to),
                    };
                    let id = analyzer.declared(to.as_ref()).unwrap();
                    let definition = definitions.get_mut(id).unwrap();
                    let DefinitionItem::Alias(alias) = &mut definition.item else { unreachable!() };
                    alias.value = Some(
                        Expression::builtin(from_span, Builtin::Access)
                            .apply_to(from.span(), expression.clone())
                            .apply_to(to.span(), Expression::dynamic(to.clone())),
                    );
                }
            }
            syntax::DefinitionItem::Module(ast) => {
                let id = analyzer.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(id).unwrap();
                let DefinitionItem::Module(module) = &mut definition.item else { unreachable!() };
                module.module = Some(Module::convert_module(analyzer, *ast));
            }
            syntax::DefinitionItem::ModuleImport(ast) => {
                let expression = Expression::convert_module_path(analyzer, ast.module);
                let id = analyzer.declared(ast.name.as_ref()).unwrap();
                let definition = definitions.get_mut(id).unwrap();
                let DefinitionItem::Alias(alias) = &mut definition.item else { unreachable!() };
                alias.value = Some(expression);
            }
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

    pub(super) fn declare(analyzer: &mut Analyzer, ast: &syntax::Definition) -> Vec<Self> {
        let def = match &ast.item {
            syntax::DefinitionItem::Export(..) => return vec![],
            syntax::DefinitionItem::ExternalModule(ast) => {
                Identifier::declare(analyzer, ast.head.name.clone());
                return vec![];
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
                return ast
                    .names
                    .iter()
                    .map(|alias| {
                        let span = alias.span();
                        let name = match alias {
                            syntax::Alias::Same(name) => {
                                Identifier::declare(analyzer, name.clone())
                            }
                            syntax::Alias::Rename(_, name) => {
                                Identifier::declare(analyzer, name.clone())
                            }
                        };
                        Self {
                            span,
                            item: DefinitionItem::Alias(Box::new(Alias::declare(name))),
                            is_exported: false,
                        }
                    })
                    .collect();
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
                let name = Identifier::declare(analyzer, ast.name.clone());
                Self {
                    span: ast.span(),
                    item: DefinitionItem::Alias(Box::new(Alias::declare(name))),
                    is_exported: false,
                }
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
            syntax::DefinitionItem::Test(..) => return vec![],
        };

        vec![def]
    }
}
