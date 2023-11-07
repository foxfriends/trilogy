use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct TestDefinition {
    pub test: Token,
    pub not: Option<Token>,
    pub name: StringLiteral,
    pub body: Block,
}

impl Spanned for TestDefinition {
    fn span(&self) -> Span {
        self.test.span.union(self.body.span())
    }
}

impl TestDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let test = parser
            .expect(TokenType::KwTest)
            .expect("Caller should find `test` keyword.");
        let not = parser.expect(TokenType::KwNot).ok();
        let name = StringLiteral::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self {
            test,
            not,
            name,
            body,
        })
    }
}
