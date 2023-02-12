use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct StructLiteral {
    pub atom: AtomLiteral,
    pub value: Expression,
    end: Token,
}

impl StructLiteral {
    pub(crate) fn parse(parser: &mut Parser, atom: AtomLiteral) -> SyntaxResult<Self> {
        parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let value = Expression::parse(parser)?;
        let end = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(Self { atom, value, end })
    }
}
