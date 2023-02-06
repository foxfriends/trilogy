use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned)]
pub struct StructLiteral {
    pub name: AtomLiteral,
    pub value: Expression,
    end: Token,
}

impl StructLiteral {
    pub(crate) fn parse(parser: &mut Parser, name: AtomLiteral) -> SyntaxResult<Self> {
        parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let value = Expression::parse(parser)?;
        let end = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(Self { name, value, end })
    }
}
