use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ArrayComprehension {
    start: Token,
    pub expression: Expression,
    pub query: Query,
    end: Token,
}

impl ArrayComprehension {
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        expression: Expression,
    ) -> SyntaxResult<Self> {
        let query = Query::parse(parser)?;
        let end = parser
            .expect(CBrack)
            .map_err(|token| parser.expected(token, "expected `]` to end array comprehension"))?;
        Ok(Self {
            start,
            expression,
            query,
            end,
        })
    }
}
