use super::{query::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct QueryImplication {
    pub r#if: Token,
    pub lhs: Query,
    pub then: Token,
    pub rhs: Query,
    pub span: Span,
}

impl Spanned for QueryImplication {
    fn span(&self) -> Span {
        self.span
    }
}

impl QueryImplication {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#if = parser.expect(KwIf).unwrap();
        let lhs = Query::parse(parser)?;
        let then = parser.expect(KwThen).map_err(|token| {
            parser.expected(token, "expected `then` to follow implication query")
        })?;
        let rhs = Query::parse_precedence(parser, Precedence::Implication)?;
        Ok(Self {
            span: r#if.span.union(rhs.span()),
            r#if,
            lhs,
            then,
            rhs,
        })
    }
}
