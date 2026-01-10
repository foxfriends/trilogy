use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct IsExpression {
    pub is: Token,
    pub query: Query,
    pub span: Span,
}

impl Spanned for IsExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl IsExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let is = parser.expect(KwIs).expect("Caller should have found this");
        let query = Query::parse(parser)?;
        Ok(Self {
            span: is.span.union(query.span()),
            is,
            query,
        })
    }
}
