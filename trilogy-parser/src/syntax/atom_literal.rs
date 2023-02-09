use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct AtomLiteral {
    token: Token,
}

impl AtomLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Atom)
            .map_err(|token| parser.expected(token, "expected atom literal"))?;
        Ok(Self { token })
    }
}
