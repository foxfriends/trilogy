use super::*;
use crate::{Converter, Error, Id};
use source_span::Span;
use std::sync::Arc;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Definition {
    pub span: Span,
    pub item: DefinitionItem,
    pub is_exported: bool,
    pub export_span: Option<Span>,
}

impl Definition {
    fn new(span: Span, item: impl Into<DefinitionItem>) -> Self {
        Self {
            span,
            item: item.into(),
            is_exported: false,
            export_span: None,
        }
    }

    pub fn name(&self) -> Option<&Id> {
        self.item.name()
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
                        Some(id) => {
                            let def = definitions.get_mut(id).unwrap();
                            if def.is_exported {
                                converter.error(Error::DuplicateExport {
                                    original: def.export_span.unwrap(),
                                    duplicate: name.clone(),
                                });
                            }
                            def.is_exported = true;
                            def.export_span = Some(name.span());
                        }
                        None => {
                            converter.error(Error::UnknownExport { name: name.clone() });
                        }
                    }
                }
            }
            syntax::DefinitionItem::Constant(ast) => {
                let symbol = converter.declared(ast.name.as_ref()).unwrap();
                let definition = definitions.get_mut(symbol).unwrap();
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
            syntax::DefinitionItem::Function(ast) => {
                let symbol = converter.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(symbol).unwrap();
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
            syntax::DefinitionItem::ExternalModule(ast) => {
                if let Some(module_use) = ast.module_use {
                    let module_symbol = converter.declared(ast.head.name.as_ref()).unwrap().clone();
                    let module_ident = Identifier::declared(converter, &ast.head.name).unwrap();
                    let module_definition = definitions.get_mut(&module_symbol).unwrap();
                    let DefinitionItem::Module(..) = &mut module_definition.item else {
                        let error = Error::DuplicateDefinition {
                            original: module_symbol.declaration_span,
                            duplicate: ast.head.name,
                        };
                        converter.error(error);
                        return;
                    };

                    for name in module_use.names {
                        let symbol = converter.declared(name.as_ref()).unwrap().clone();
                        let definition = definitions.get_mut(&symbol).unwrap();
                        let DefinitionItem::Constant(constant) = &mut definition.item else {
                            let error = Error::DuplicateDefinition {
                                original: symbol.declaration_span,
                                duplicate: name,
                            };
                            converter.error(error);
                            return;
                        };
                        constant.value = Expression::module_access(
                            module_use.r#use.span,
                            Expression::reference(
                                module_symbol.declaration_span,
                                module_ident.clone(),
                            ),
                            name,
                        )
                    }
                }
            }
            syntax::DefinitionItem::Module(ast) => {
                let module_symbol = converter.declared(ast.head.name.as_ref()).unwrap().clone();

                if let Some(module_use) = &ast.module_use {
                    let module_ident = Identifier::declared(converter, &ast.head.name).unwrap();
                    for name in &module_use.names {
                        let symbol = converter.declared(name.as_ref()).unwrap().clone();
                        let definition = definitions.get_mut(&symbol).unwrap();
                        let DefinitionItem::Constant(constant) = &mut definition.item else {
                            let error = Error::DuplicateDefinition {
                                original: symbol.declaration_span,
                                duplicate: name.clone(),
                            };
                            converter.error(error);
                            return;
                        };
                        constant.value = Expression::module_access(
                            module_use.r#use.span,
                            Expression::reference(
                                module_symbol.declaration_span,
                                module_ident.clone(),
                            ),
                            name.clone(),
                        )
                    }
                }

                let module_definition = definitions.get_mut(&module_symbol).unwrap();
                let DefinitionItem::Module(module) = &mut module_definition.item else {
                    let error = Error::DuplicateDefinition {
                        original: module_symbol.declaration_span,
                        duplicate: ast.head.name,
                    };
                    converter.error(error);
                    return;
                };

                module.module = Arc::new(ModuleCell::new(Module::convert_module(converter, *ast)));
            }
            syntax::DefinitionItem::Procedure(ast) => {
                let symbol = converter.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(symbol).unwrap();
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
            syntax::DefinitionItem::ExternalProcedure(..) => {}
            syntax::DefinitionItem::Rule(ast) => {
                let symbol = converter.declared(ast.head.name.as_ref()).unwrap();
                let definition = definitions.get_mut(symbol).unwrap();
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
                definitions.push(Self::new(
                    ast.span(),
                    TestDefinition::convert(converter, *ast),
                ));
            }
        }
    }

    pub(super) fn declare(converter: &mut Converter, ast: &syntax::Definition) -> Vec<Self> {
        let def = match &ast.item {
            syntax::DefinitionItem::Export(..) => return vec![],
            syntax::DefinitionItem::Constant(ast) => {
                if let Some(original) = converter.declared_no_shadow(ast.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.name.clone(),
                    });
                    return vec![];
                }
                let name = Identifier::declare(converter, ast.name.clone());
                Self::new(ast.span(), ConstantDefinition::declare(name))
            }
            syntax::DefinitionItem::ExternalModule(ast) => {
                if let Some(original) = converter.declared_no_shadow(ast.head.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.head.name.clone(),
                    });
                    return vec![];
                }

                let name = Identifier::declare(converter, ast.head.name.clone());
                let mut names = vec![Self::new(
                    ast.span(),
                    ModuleDefinition::external(name, converter.resolve(&ast.locator.value())),
                )];

                if let Some(module_use) = &ast.module_use {
                    for name in &module_use.names {
                        if let Some(original) = converter.declared_no_shadow(name.as_ref()) {
                            let original = original.declaration_span;
                            converter.error(Error::DuplicateDefinition {
                                original,
                                duplicate: ast.head.name.clone(),
                            });
                        }

                        let used_name = Identifier::declare(converter, name.clone());
                        names.push(Self::new(
                            name.span(),
                            ConstantDefinition::declare(used_name),
                        ));
                    }
                }

                return names;
            }
            syntax::DefinitionItem::Function(ast) => {
                if converter
                    .declared_no_shadow(ast.head.name.as_ref())
                    .is_some()
                {
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self::new(span, FunctionDefinition::declare(name))
            }
            syntax::DefinitionItem::Module(ast) => {
                if let Some(original) = converter.declared_no_shadow(ast.head.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self::new(ast.span(), ModuleDefinition::declare(name))
            }
            syntax::DefinitionItem::Procedure(ast) => {
                if let Some(original) = converter.declared_no_shadow(ast.head.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self::new(
                    span,
                    ProcedureDefinition::declare(name, ast.head.parameters.len()),
                )
            }
            syntax::DefinitionItem::ExternalProcedure(ast) => {
                if let Some(original) = converter.declared_no_shadow(ast.head.name.as_ref()) {
                    let original = original.declaration_span;
                    converter.error(Error::DuplicateDefinition {
                        original,
                        duplicate: ast.head.name.clone(),
                    });
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self::new(
                    span,
                    ProcedureDefinition::declare(name, ast.head.parameters.len()),
                )
            }
            syntax::DefinitionItem::Rule(ast) => {
                if converter
                    .declared_no_shadow(ast.head.name.as_ref())
                    .is_some()
                {
                    return vec![];
                }
                let span = ast.span();
                let name = Identifier::declare(converter, ast.head.name.clone());
                Self::new(span, RuleDefinition::declare(name))
            }
            syntax::DefinitionItem::Test(..) => return vec![],
        };

        vec![def]
    }
}
