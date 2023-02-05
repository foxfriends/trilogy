use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StringLiteral {
    token: Token,
}

impl StringLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser.expect(TokenType::String).map_err(|token| {
            let error = SyntaxError::new(token.span, "expected string literal");
            parser.error(error.clone());
            error
        })?;
        Ok(Self { token })
    }
}
