use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct RecordComprehension {
    start: Token,
    pub key_expression: Expression,
    pub expression: Expression,
    pub query: Query,
    end: Token,
}

impl RecordComprehension {
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        key_expression: Expression,
        expression: Expression,
    ) -> SyntaxResult<Self> {
        let query = Query::parse(parser)?;
        let end = parser
            .expect(CBracePipe)
            .map_err(|token| parser.expected(token, "expected `|}` to end record comprehension"))?;
        Ok(Self {
            start,
            key_expression,
            expression,
            query,
            end,
        })
    }
}
