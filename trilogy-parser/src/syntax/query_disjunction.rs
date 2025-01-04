use super::{query::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct QueryDisjunction {
    pub lhs: Query,
    pub or: Token,
    pub rhs: Query,
    span: Span,
}

impl Spanned for QueryDisjunction {
    fn span(&self) -> Span {
        self.span
    }
}

impl QueryDisjunction {
    pub(crate) fn parse(parser: &mut Parser, lhs: Query) -> SyntaxResult<Self> {
        let or = parser.expect(KwOr).unwrap();
        let rhs = Query::parse_precedence(parser, Precedence::Disjunction)?;
        Ok(Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            or,
            rhs,
        })
    }
}
