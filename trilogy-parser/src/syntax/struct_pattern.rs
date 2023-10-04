use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct StructPattern {
    pub atom: AtomLiteral,
    pub start: Token,
    pub pattern: Pattern,
    pub end: Token,
}

impl StructPattern {
    pub(crate) fn parse(parser: &mut Parser, atom: AtomLiteral) -> SyntaxResult<Self> {
        let start = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let pattern = Pattern::parse(parser)?;
        let end = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(Self {
            atom,
            start,
            pattern,
            end,
        })
    }
}

impl TryFrom<StructLiteral> for StructPattern {
    type Error = SyntaxError;

    fn try_from(value: StructLiteral) -> Result<Self, Self::Error> {
        Ok(Self {
            atom: value.atom,
            start: value.start,
            pattern: value.value.try_into()?,
            end: value.end,
        })
    }
}
