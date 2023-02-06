use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct NumberLiteral {
    token: Token,
}

impl NumberLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Numeric)
            .map_err(|token| parser.expected(token, "expected number literal"))?;
        Ok(Self { token })
    }
}
