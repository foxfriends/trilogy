use super::{query::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct QueryAlternative {
    pub lhs: Query,
    pub r#else: Token,
    pub rhs: Query,
    span: Span,
}

impl Spanned for QueryAlternative {
    fn span(&self) -> Span {
        self.span
    }
}

impl QueryAlternative {
    pub(crate) fn parse(parser: &mut Parser, lhs: Query) -> SyntaxResult<Self> {
        let r#else = parser.expect(KwElse).unwrap();
        let rhs = Query::parse_precedence(parser, Precedence::Disjunction)?;
        Ok(Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            r#else,
            rhs,
        })
    }
}
