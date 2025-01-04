use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct IteratorComprehension {
    pub open_dollar_paren: Token,
    pub expression: Expression,
    pub query: Query,
    pub close_paren: Token,
}

impl Spanned for IteratorComprehension {
    fn span(&self) -> Span {
        self.open_dollar_paren.span.union(self.close_paren.span)
    }
}

impl IteratorComprehension {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let open_dollar_paren = parser.expect(DollarOParen).unwrap();
        let expression = Expression::parse(parser)?;
        parser.expect(KwFor).map_err(|token| {
            parser.expected(
                token,
                "expected `for` to follow the expression of an iterator comprehension",
            )
        })?;
        let query = Query::parse(parser)?;
        let close_paren = parser.expect(CParen).map_err(|token| {
            parser.expected(token, "expected `)` to end iterator comprehension")
        })?;
        Ok(Self {
            open_dollar_paren,
            expression,
            query,
            close_paren,
        })
    }
}
