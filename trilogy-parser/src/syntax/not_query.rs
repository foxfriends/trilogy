use super::{query::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct NotQuery {
    pub not: Token,
    pub query: Query,
    pub span: Span,
}

impl Spanned for NotQuery {
    fn span(&self) -> Span {
        self.span
    }
}

impl NotQuery {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let not = parser.expect(KwNot).unwrap();
        let query = Query::parse_precedence(parser, Precedence::Not)?;
        Ok(Self {
            span: not.span.union(query.span()),
            not,
            query,
        })
    }
}
