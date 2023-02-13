use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BreakStatement {
    token: Token,
}

impl BreakStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::KwBreak)
            .expect("Caller should have found this");
        Ok(Self { token })
    }
}
