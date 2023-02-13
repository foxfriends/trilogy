use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct EndStatement {
    token: Token,
}

impl EndStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::KwEnd)
            .expect("Caller should have found this");
        Ok(Self { token })
    }
}
