use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct IsExpression {
    start: Token,
    pub query: Query,
}

impl IsExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwIs).expect("Caller should have found this");
        let query = Query::parse(parser)?;
        Ok(Self { start, query })
    }
}
