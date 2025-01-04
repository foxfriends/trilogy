use super::{pattern::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct PatternDisjunction {
    pub lhs: Pattern,
    pub or: Token,
    pub rhs: Pattern,
    span: Span,
}

impl Spanned for PatternDisjunction {
    fn span(&self) -> Span {
        self.span
    }
}

impl PatternDisjunction {
    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        let or = parser.expect(KwOr).unwrap();
        let rhs = Pattern::parse_precedence(parser, Precedence::Disjunction)?;
        Ok(Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            or,
            rhs,
        })
    }
}
