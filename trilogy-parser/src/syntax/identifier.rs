use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Identifier {
    token: Token,
}

impl Identifier {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser.expect(TokenType::Identifier).map_err(|token| {
            let error = SyntaxError::new(token.span, "expected identifier");
            parser.error(error.clone());
            error
        })?;
        Ok(Self { token })
    }
}
