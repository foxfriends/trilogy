use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TestDefinition {
    start: Token,
    pub name: StringLiteral,
    pub body: Block,
}

impl TestDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwTest)
            .expect("Caller should find `test` keyword.");
        let name = StringLiteral::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self { start, name, body })
    }
}
