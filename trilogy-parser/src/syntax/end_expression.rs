use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct EndExpression {
    token: Token,
}

impl EndExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser.expect(KwEnd).expect("Caller should have found this");
        Ok(Self { token })
    }
}
