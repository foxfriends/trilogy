use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BitsLiteral {
    token: Token,
}

impl BitsLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Bits)
            .map_err(|token| parser.expected(token, "expected bits literal"))?;
        Ok(Self { token })
    }
}
