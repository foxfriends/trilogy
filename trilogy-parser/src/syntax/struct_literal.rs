use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct StructLiteral {
    pub atom: AtomLiteral,
    pub start: Token,
    pub value: Expression,
    pub end: Token,
}

impl StructLiteral {
    pub(crate) fn parse(
        parser: &mut Parser,
        atom: AtomLiteral,
    ) -> SyntaxResult<Result<Self, StructPattern>> {
        let start = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let value = Expression::parse_or_pattern(parser)?;
        let end = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;

        match value {
            Ok(value) => Ok(Ok(Self {
                atom,
                start,
                value,
                end,
            })),
            Err(pattern) => Ok(Err(StructPattern {
                atom,
                start,
                pattern,
                end,
            })),
        }
    }
}
