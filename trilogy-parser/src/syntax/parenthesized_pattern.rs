use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ParenthesizedPattern {
    start: Token,
    pub pattern: Pattern,
    end: Token,
}

impl ParenthesizedPattern {
    pub(crate) fn finish(
        parser: &mut Parser,
        start: Token,
        pattern: Pattern,
    ) -> SyntaxResult<Self> {
        let end = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(Self {
            start,
            pattern,
            end,
        })
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let pattern = Pattern::parse(parser)?;
        Self::finish(parser, start, pattern)
    }
}
