use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct SetComprehension {
    start: Token,
    pub expression: Expression,
    pub query: Query,
    end: Token,
}

impl SetComprehension {
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        expression: Expression,
    ) -> SyntaxResult<Self> {
        let query = Query::parse(parser)?;
        let end = parser
            .expect(CBrackPipe)
            .map_err(|token| parser.expected(token, "expected `|]` to end set comprehension"))?;
        Ok(Self {
            start,
            expression,
            query,
            end,
        })
    }

    pub fn start_token(&self) -> &Token {
        &self.start
    }

    pub fn end_token(&self) -> &Token {
        &self.end
    }
}
