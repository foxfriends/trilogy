use super::*;
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ModulePath {
    pub modules: Vec<ModuleReference>,
}

impl ModulePath {
    pub(crate) fn parse_rest(parser: &mut Parser, first: ModuleReference) -> SyntaxResult<Self> {
        let mut modules = vec![first];
        loop {
            if parser.expect(OpColonColon).is_err() {
                break;
            }
            modules.push(ModuleReference::parse(parser)?);
        }
        Ok(Self { modules })
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let first = ModuleReference::parse(parser)?;
        Self::parse_rest(parser, first)
    }
}
