use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BooleanLiteral {
    token: Token,
}

impl BooleanLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect([TokenType::KwTrue, TokenType::KwFalse])
            .map_err(|token| parser.expected(token, "expected boolean literal"))?;
        Ok(Self { token })
    }
}
