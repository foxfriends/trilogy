use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug)]
pub struct ExternalModuleDefinition {
    pub head: ModuleHead,
    pub locator: StringLiteral,
}

impl ExternalModuleDefinition {
    pub(crate) fn parse(parser: &mut Parser, head: ModuleHead) -> SyntaxResult<Self> {
        parser
            .expect(TokenType::KwAt)
            .expect("Caller should find `at` keyword.");
        let locator = StringLiteral::parse(parser)?;
        Ok(Self { head, locator })
    }
}
