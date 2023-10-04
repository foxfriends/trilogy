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
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let expression = Expression::parse(parser)?;
        let end = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(Self {
            start,
            expression,
            end,
        })
    }
}
