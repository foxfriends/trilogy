use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ParenthesizedExpression {
    pub open_paren: Token,
    pub expression: Expression,
    pub close_paren: Token,
}

impl Spanned for ParenthesizedExpression {
    fn span(&self) -> Span {
        self.open_paren.span.union(self.close_paren.span())
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
        Ok(match expression {
            Ok(expression) => Ok(Self {
                open_paren,
                expression,
                close_paren,
            }),
            Err(pattern) => Err(ParenthesizedPattern {
                open_paren,
                pattern,
                close_paren,
            }),
        })
    }
}
