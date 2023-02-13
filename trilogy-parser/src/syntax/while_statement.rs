use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct WhileStatement {
    start: Token,
    pub condition: Expression,
    pub body: Block,
}

impl WhileStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwWhile)
            .expect("Caller should have found this");
        let condition = Expression::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self {
            start,
            condition,
            body,
        })
    }
}
