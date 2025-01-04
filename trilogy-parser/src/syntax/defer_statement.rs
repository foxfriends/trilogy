use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct DeferStatement {
    pub defer: Token,
    pub body: Block,
}

impl DeferStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let defer = parser
            .expect(TokenType::KwDefer)
            .expect("Caller should have found this");
        let body = Block::parse(parser)?;
        Ok(Self { defer, body })
    }
}
