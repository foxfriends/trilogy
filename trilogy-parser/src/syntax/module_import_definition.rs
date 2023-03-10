use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ModuleImportDefinition {
    start: Token,
    pub module: ModulePath,
    pub name: Identifier,
}

impl ModuleImportDefinition {
    pub(crate) fn parse(parser: &mut Parser, start: Token) -> SyntaxResult<Self> {
        let module = ModulePath::parse(parser)?;
        parser
            .expect(TokenType::KwAs)
            .map_err(|token| parser.expected(token, "expected keyword `as`"))?;
        let name = Identifier::parse(parser)?;
        Ok(Self {
            start,
            module,
            name,
        })
    }
}
