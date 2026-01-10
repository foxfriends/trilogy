use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct ParenthesizedExpression {
    pub open_paren: Token,
    pub expression: Expression,
    pub close_paren: Token,
    pub span: Span,
}

impl Spanned for ParenthesizedExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl ParenthesizedExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Result<Self, ParenthesizedPattern>> {
        let open_paren = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let expression = Expression::parse_or_pattern(parser)?;
        let close_paren = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        let span = open_paren.span.union(close_paren.span);
        Ok(match expression {
            Ok(expression) => Ok(Self {
                span,
                open_paren,
                expression,
                close_paren,
            }),
            Err(pattern) => Err(ParenthesizedPattern {
                span,
                open_paren,
                pattern,
                close_paren,
            }),
        })
    }
}
