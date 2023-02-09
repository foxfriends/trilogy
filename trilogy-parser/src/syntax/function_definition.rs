use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct FunctionDefinition {
    start: Token,
    pub head: FunctionHead,
    pub body: Expression,
}

impl FunctionDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwFunc)
            .expect("Caller should find `func` keyword.");
        let head = FunctionHead::parse(parser)?;
        let body = Expression::parse(parser)?;
        Ok(FunctionDefinition { start, head, body })
    }
}
