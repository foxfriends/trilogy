use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct CharacterLiteral {
    token: Token,
}

impl CharacterLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Character)
            .map_err(|token| parser.expected(token, "expected character literal"))?;
        Ok(Self { token })
    }
}
