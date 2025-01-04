use super::{pattern::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct PatternConjunction {
    pub lhs: Pattern,
    pub and: Token,
    pub rhs: Pattern,
    span: Span,
}

impl Spanned for PatternConjunction {
    fn span(&self) -> Span {
        self.span
    }
}

impl PatternConjunction {
    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        let and = parser.expect(KwAnd).unwrap();
        let rhs = Pattern::parse_precedence(parser, Precedence::Conjunction)?;
        Ok(Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            and,
            rhs,
        })
    }
}
