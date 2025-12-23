use super::*;
use crate::{Converter, Error, Id, visitor::MightBeConstant};
use source_span::Span;
use std::{collections::HashMap, sync::Arc};
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
        anonymous: &HashMap<Span, Identifier>,
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
                if !constant.value.is_constant() {
                    converter.error(Error::NonConstantExpressionInConstant {
                        expression: constant.value.span,
                    });
                }
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
            syntax::DefinitionItem::Import(import) => {
                let module_ident = if let Some(type_as) = import.type_as {
                    let ident = Identifier::declared(converter, &type_as.identifier).unwrap();
                    let module_definition = definitions.get_mut(&ident.id).unwrap();
                    let DefinitionItem::Module(..) = &mut module_definition.item else {
                        let error = Error::DuplicateDefinition {
                            original: ident.id.declaration_span,
                            duplicate: type_as.identifier,
                        };
                        converter.error(error);
                        return;
                    };
                    Some(ident)
                } else {
                    None
                };
                if let Some(type_use) = import.type_use {
                    // NOTE: use the created one from below
                    let module_ident = module_ident
                        .unwrap_or_else(|| anonymous.get(&import.import.span).unwrap().clone());

                    for name in type_use.names {
                        let symbol = converter
                            .declared(name.aliased_name().as_ref())
                            .unwrap()
                            .clone();
                        let definition = definitions.get_mut(&symbol).unwrap();
                        let DefinitionItem::Constant(constant) = &mut definition.item else {
                            let error = Error::DuplicateDefinition {
                                original: symbol.declaration_span,
                                duplicate: name.aliased_name().clone(),
                            };
                            converter.error(error);
                            return;
                        };
                        constant.value = Expression::module_access(
                            type_use.r#use.span,
                            Expression::reference(
                                module_ident.id.declaration_span,
                                module_ident.clone(),
                            ),
                            name.original_name().clone(),
                        )
                    }
                }
            }
            syntax::DefinitionItem::Type(ast) => {
                let module_symbol = converter.declared(ast.head.name.as_ref()).unwrap().clone();

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

    pub(super) fn declare(
        converter: &mut Converter,
        ast: &syntax::Definition,
        anonymous: &mut HashMap<Span, Identifier>,
    ) -> Vec<Self> {
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
            syntax::DefinitionItem::Import(import) => {
                let name = if let Some(type_as) = &import.type_as {
                    if let Some(original) =
                        converter.declared_no_shadow(type_as.identifier.as_ref())
                    {
                        let original = original.declaration_span;
                        converter.error(Error::DuplicateDefinition {
                            original,
                            duplicate: type_as.identifier.clone(),
                        });
                        return vec![];
                    }
                    Identifier::declare(converter, type_as.identifier.clone())
                } else {
                    let ident = Identifier::temporary(converter, import.locator.span());
                    anonymous.insert(import.import.span, ident.clone());
                    ident
                };

                let mut names = vec![Self::new(
                    import.span(),
                    ModuleDefinition::external(name, converter.resolve(&import.locator.value())),
                )];

                if let Some(module_use) = &import.type_use {
                    for name in &module_use.names {
                        if let Some(original) =
                            converter.declared_no_shadow(name.aliased_name().as_ref())
                        {
                            let original = original.declaration_span;
                            converter.error(Error::DuplicateDefinition {
                                original,
                                duplicate: name.aliased_name().clone(),
                            });
                        }

                        let used_name = Identifier::declare(converter, name.aliased_name().clone());
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
            syntax::DefinitionItem::Type(ast) => {
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
