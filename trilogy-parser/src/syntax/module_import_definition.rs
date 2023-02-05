use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct ModuleImportDefinition {
    start: Token,
    pub module: ModulePath,
    pub name: Identifier,
}

impl ModuleImportDefinition {
    pub(crate) fn parse(
        parser: &mut Parser,
        start: Token,
        first: ModuleReference,
    ) -> SyntaxResult<Self> {
        let module = ModulePath::parse_rest(parser, first)?;
        parser.expect(TokenType::KwAs).map_err(|token| {
            let error = SyntaxError::new(token.span, "expected keyword `as`");
            parser.error(error.clone());
            error
        })?;
        let name = Identifier::parse(parser)?;
        Ok(Self {
            start,
            module,
            name,
        })
    }
}
