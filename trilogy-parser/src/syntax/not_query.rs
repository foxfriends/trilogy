use super::{query::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct NotQuery {
    start: Token,
    pub query: Query,
}

impl NotQuery {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwNot).expect("Caller should have found this");
        let query = Query::parse_precedence(parser, Precedence::Not)?;
        Ok(Self { start, query })
    }
}
