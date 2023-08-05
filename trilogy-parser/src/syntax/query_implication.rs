use super::{query::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct QueryImplication {
    start: Token,
    pub lhs: Query,
    then: Token,
    pub rhs: Query,
}

impl QueryImplication {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwIf).expect("Caller should have found this");
        let lhs = Query::parse(parser)?;
        let then = parser.expect(KwThen).map_err(|token| {
            parser.expected(token, "expected `then` to follow implication query")
        })?;
        let rhs = Query::parse_precedence(parser, Precedence::Implication)?;
        Ok(Self {
            start,
            lhs,
            then,
            rhs,
        })
    }
}
