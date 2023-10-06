use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ParenthesizedExpression {
    pub start: Token,
    pub expression: Expression,
    pub end: Token,
}

impl ParenthesizedExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Result<Self, ParenthesizedPattern>> {
        let start = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let expression = Expression::parse_or_pattern(parser)?;
        let end = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(match expression {
            Ok(expression) => Ok(Self {
                start,
                expression,
                end,
            }),
            Err(pattern) => Err(ParenthesizedPattern {
                start,
                pattern,
                end,
            }),
        })
    }
}
