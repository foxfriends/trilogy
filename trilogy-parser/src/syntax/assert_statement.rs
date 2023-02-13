use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct AssertStatement {
    start: Token,
    pub message: Option<Expression>,
    pub assertion: Expression,
}

impl AssertStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwAssert)
            .expect("Caller should have found this");
        let mut message = Some(Expression::parse(parser)?);
        let assertion = parser
            .expect(TokenType::KwAs)
            .ok()
            .map(|_| Expression::parse(parser))
            .transpose()?
            .unwrap_or_else(|| message.take().unwrap());
        Ok(Self {
            start,
            message,
            assertion,
        })
    }
}

impl Spanned for AssertStatement {
    fn span(&self) -> Span {
        self.start.span.union(self.assertion.span())
    }
}
