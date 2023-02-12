use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct StructPattern {
    pub atom: AtomLiteral,
    pub pattern: Pattern,
    end: Token,
}

impl StructPattern {
    pub(crate) fn parse(parser: &mut Parser, atom: AtomLiteral) -> SyntaxResult<Self> {
        parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let pattern = Pattern::parse(parser)?;
        let end = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(Self { atom, pattern, end })
    }
}
