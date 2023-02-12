use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct IteratorComprehension {
    start: Token,
    pub expression: Expression,
    pub query: Query,
    end: Token,
}

impl IteratorComprehension {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(DollarOParen)
            .expect("Caller should have found this");
        let expression = Expression::parse(parser)?;
        parser.expect(KwFor).map_err(|token| {
            parser.expected(
                token,
                "expected `for` to follow the expression of an iterator comprehension",
            )
        })?;
        let query = Query::parse(parser)?;
        let end = parser.expect(CParen).map_err(|token| {
            parser.expected(token, "expected `)` to end iterator comprehension")
        })?;
        Ok(Self {
            start,
            expression,
            query,
            end,
        })
    }
}
