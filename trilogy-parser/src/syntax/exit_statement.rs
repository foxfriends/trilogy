use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ExitStatement {
    start: Token,
    pub expression: Expression,
}

impl ExitStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwExit)
            .expect("Caller should have found this");
        let expression = Expression::parse(parser)?;
        Ok(Self { start, expression })
    }
}
