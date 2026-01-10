use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct ParenthesizedPattern {
    pub open_paren: Token,
    pub pattern: Pattern,
    pub close_paren: Token,
    pub span: Span,
}

impl Spanned for ParenthesizedPattern {
    fn span(&self) -> Span {
        self.span
    }
}

impl ParenthesizedPattern {
    pub(crate) fn finish(
        parser: &mut Parser,
        open_paren: Token,
        pattern: Pattern,
    ) -> SyntaxResult<Self> {
        let close_paren = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(Self {
            span: open_paren.span.union(close_paren.span()),
            open_paren,
            pattern,
            close_paren,
        })
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let open_paren = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let pattern = Pattern::parse(parser)?;
        Self::finish(parser, open_paren, pattern)
    }
}

impl TryFrom<ParenthesizedExpression> for ParenthesizedPattern {
    type Error = SyntaxError;

    fn try_from(value: ParenthesizedExpression) -> Result<Self, Self::Error> {
        Ok(Self {
            span: value.span,
            open_paren: value.open_paren,
            pattern: value.expression.try_into()?,
            close_paren: value.close_paren,
        })
    }
}
