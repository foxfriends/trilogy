use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug)]
pub enum DefinitionItem {
    Module(Box<ModuleDefinition>),
    ExternalModule(Box<ExternalModuleDefinition>),
    Procedure(Box<ProcedureDefinition>),
    Function(Box<FunctionDefinition>),
    Rule(Box<RuleDefinition>),
    Import(Box<ImportDefinition>),
    ModuleImport(Box<ModuleImportDefinition>),
    Export(Box<ExportDefinition>),
    Test(Box<TestDefinition>),
}

#[derive(Clone, Debug)]
pub struct Definition {
    pub documentation: Option<Documentation>,
    pub item: DefinitionItem,
}

impl Definition {
    pub(crate) fn try_parse(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        let documentation = Documentation::parse_outer(parser);
        let token = parser.peek();
        match token.token_type {
            EndOfFile if documentation.is_some() => {
                let error = SyntaxError::new(
                    documentation.as_ref().unwrap().span(),
                    "outer documentation comment must precede the item it documents",
                );
                parser.error(error.clone());
                Err(error)
            }
            EndOfFile => Ok(None),
            KwModule => {
                let _head = ModuleHead::parse(parser)?;
                let token = parser.peek();
                match token.token_type {
                    KwAt => todo!(),   // Ok(Some(ExternalModuleDefinition::parse(parser, head)?)),
                    OBrace => todo!(), // Ok(Some(ModuleDefinition::parse(parser, head)?)),
                    _ => {
                        let error = SyntaxError::new(
                            token.span,
                            "expected `at` for an external module, or { for a local module",
                        );
                        parser.error(error.clone());
                        Err(error)
                    }
                }
            }
            _ => {
                let error = SyntaxError::new(token.span, "unexpected token at top level");
                parser.error(error.clone());
                Err(error)
            }
        }
    }
}
