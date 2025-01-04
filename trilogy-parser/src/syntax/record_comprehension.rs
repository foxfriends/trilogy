use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RecordComprehension {
    pub open_brace_pipe: Token,
    pub key_expression: Expression,
    pub expression: Expression,
    pub query: Query,
    pub close_brace_pipe: Token,
}

impl Spanned for RecordComprehension {
    fn span(&self) -> source_span::Span {
        self.open_brace_pipe.span.union(self.close_brace_pipe.span)
    }
}

impl RecordComprehension {
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        open_brace_pipe: Token,
        key_expression: Expression,
        expression: Expression,
    ) -> SyntaxResult<Self> {
        let query = Query::parse(parser)?;
        let close_brace_pipe = parser
            .expect(CBracePipe)
            .map_err(|token| parser.expected(token, "expected `|}` to end record comprehension"))?;
        Ok(Self {
            open_brace_pipe,
            key_expression,
            expression,
            query,
            close_brace_pipe,
        })
    }
}
