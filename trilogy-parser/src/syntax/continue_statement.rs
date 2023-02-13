use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ContinueStatement {
    token: Token,
}

impl ContinueStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::KwContinue)
            .expect("Caller should have found this");
        Ok(Self { token })
    }
}
