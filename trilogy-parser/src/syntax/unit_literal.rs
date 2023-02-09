use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct UnitLiteral {
    token: Token,
}

impl UnitLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::KwUnit)
            .map_err(|token| parser.expected(token, "expected boolean literal"))?;
        Ok(Self { token })
    }
}
