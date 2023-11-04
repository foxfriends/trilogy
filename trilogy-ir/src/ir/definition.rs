use std::sync::Arc;

use super::*;
use crate::{symbol::Symbol, Converter, Error, Id};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub enum DefinitionItem {
    Procedure(Box<ProcedureDefinition>),
    Function(Box<FunctionDefinition>),
    Rule(Box<RuleDefinition>),
    Test(Box<TestDefinition>),
    Constant(Box<ConstantDefinition>),
    Module(Box<ModuleDefinition>),
}

impl DefinitionItem {
    /// Returns the item if it is a procedure, or None otherwise.
    pub fn as_procedure(&self) -> Option<&ProcedureDefinition> {
        match self {
            Self::Procedure(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a procedure, or None otherwise.
    pub fn as_function(&self) -> Option<&FunctionDefinition> {
        match self {
            Self::Function(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a rule, or None otherwise
    pub fn as_rule(&self) -> Option<&RuleDefinition> {
        match self {
            Self::Rule(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a test, or None otherwise
    pub fn as_test(&self) -> Option<&TestDefinition> {
        match self {
            Self::Test(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a constant, or None otherwise
    pub fn as_constant(&self) -> Option<&ConstantDefinition> {
        match self {
            Self::Constant(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a module, or None otherwise
    pub fn as_module(&self) -> Option<&ModuleDefinition> {
        match self {
            Self::Module(def) => Some(def.as_ref()),
            _ => None,
        }
    }
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
            DefinitionItem::Constant(def) => Some(&def.name.id),
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

    pub fn is_module(&self) -> bool {
        matches!(self.item, DefinitionItem::Module(..))
    }

    pub fn is_constant(&self) -> bool {
        matches!(self.item, DefinitionItem::Constant(..))
    }

    pub(super) fn convert_into(
        converter: &mut Converter,
        ast: syntax::Definition,
        definitions: &mut Definitions,
    ) {
        match ast.item {
            syntax::DefinitionItem::Export(ast) => {
                for name in ast.names {
                    match converter.declared(name.as_ref()) {
                        Some(Symbol { id, .. }) => {
                            definitions.get_mut(id).unwrap().is_exported = true;
                        }
                        None => {
                            converter.error(Error::UnknownExport { name: name.clone() });
                        }
                    }
                }
            }
            syntax::DefinitionItem::Constant(ast) => {
                let symbol = converter.declared(ast.name.as_ref()).unwrap();
                let definition = definitions.get_mut(&symbol.id).unwrap();
                let DefinitionItem::Constant(constant) = &mut definition.item else {
                    let error = Error::DuplicateDefinition {
                        original: symbol.declaration_span,
                        duplicate: ast.name,
                    };
                    converter.error(error);
                    return;
                };
                constant.value = Expression::convert(converter, ast.body);
            }
            syntax::DefinitionItem::ExternalModule(..) => {}
            syntax::DefinitionItem::Function(ast) => {
                let symbol = converter.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(&symbol.id).unwrap();
                let DefinitionItem::Function(function) = &mut definition.item else {
                    let error = Error::DuplicateDefinition {
                        original: symbol.declaration_span,
                        duplicate: ast.head.name,
                    };
                    converter.error(error);
                    return;
                };
                function.overloads.push(Function::convert(converter, *ast))
            }
            syntax::DefinitionItem::Module(ast) => {
                let symbol = converter.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(&symbol.id).unwrap();
                let DefinitionItem::Module(module) = &mut definition.item else {
                    let error = Error::DuplicateDefinition {
                        original: symbol.declaration_span,
                        duplicate: ast.head.name,
                    };
                    converter.error(error);
                    return;
                };
                module.module = Arc::new(ModuleCell::new(Module::convert_module(converter, *ast)));
            }
            syntax::DefinitionItem::Procedure(ast) => {
                let symbol = converter.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(&symbol.id).unwrap();
                let DefinitionItem::Procedure(procedure) = &mut definition.item else {
                    let error = Error::DuplicateDefinition {
                        original: symbol.declaration_span,
                        duplicate: ast.head.name,
                    };
                    converter.error(error);
                    return;
                };
                procedure
                    .overloads
                    .push(Procedure::convert(converter, *ast))
            }
            syntax::DefinitionItem::Rule(ast) => {
                let symbol = converter.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(&symbol.id).unwrap();
                let DefinitionItem::Rule(rule) = &mut definition.item else {
                    let error = Error::DuplicateDefinition {
                        original: symbol.declaration_span,
                        duplicate: ast.head.name,
                    };
                    converter.error(error);
                    return;
                };
                rule.overloads.push(Rule::convert(converter, *ast))
            }
            syntax::DefinitionItem::Test(ast) => {
                definitions.push(Definition {
                    span: ast.span(),
                    item: DefinitionItem::Test(Box::new(TestDefinition::convert(converter, *ast))),
                    is_exported: false,
                });
            }
        }
    }

    pub(super) fn declare(converter: &mut Converter, ast: &syntax::Definition) -> Vec<Self> {
        let def = match &ast.item {
            syntax::DefinitionItem::Export(..) => return vec![],
            syntax::DefinitionItem::Constant(ast) => {
                if let Some(original) = converter.declared(ast.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.name.clone(),
                    });
                    return vec![];
                }
                let name = Identifier::declare(converter, ast.name.clone());
                Self {
                    span: ast.span(),
                    item: DefinitionItem::Constant(Box::new(ConstantDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::ExternalModule(ast) => {
                if let Some(original) = converter.declared(ast.head.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self {
                    span: ast.span(),
                    item: DefinitionItem::Module(Box::new(ModuleDefinition::external(
                        name,
                        converter.resolve(&ast.locator.value()),
                    ))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Function(ast) => {
                if converter.declared(ast.head.name.as_ref()).is_some() {
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self {
                    span,
                    item: DefinitionItem::Function(Box::new(FunctionDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Module(ast) => {
                if let Some(original) = converter.declared(ast.head.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self {
                    span: ast.span(),
                    item: DefinitionItem::Module(Box::new(ModuleDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Procedure(ast) => {
                if let Some(original) = converter.declared(ast.head.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self {
                    span,
                    item: DefinitionItem::Procedure(Box::new(ProcedureDefinition::declare(name))),
                    is_exported: false,
                }
            }
            syntax::DefinitionItem::Rule(ast) => {
                if converter.declared(ast.head.name.as_ref()).is_some() {
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(converter, ast.head.name.clone());
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
