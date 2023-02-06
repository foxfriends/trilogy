use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned)]
pub struct StringLiteral {
    token: Token,
}

impl StringLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::String)
            .map_err(|token| parser.expected(token, "expected string literal"))?;
        Ok(Self { token })
    }
}
