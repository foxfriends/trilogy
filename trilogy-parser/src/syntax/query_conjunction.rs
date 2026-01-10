use super::{query::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct QueryConjunction {
    pub lhs: Query,
    pub and: Token,
    pub rhs: Query,
    pub span: Span,
}

impl Spanned for QueryConjunction {
    fn span(&self) -> Span {
        self.span
    }
}

impl QueryConjunction {
    pub(crate) fn parse(parser: &mut Parser, lhs: Query) -> SyntaxResult<Self> {
        let and = parser.expect(KwAnd).unwrap();
        let rhs = Query::parse_precedence(parser, Precedence::Conjunction)?;
        Ok(Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            and,
            rhs,
        })
    }
}
