use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct RecordComprehension {
    pub open_brace_pipe: Token,
    pub key_expression: Expression,
    pub expression: Expression,
    pub query: Query,
    pub close_brace_pipe: Token,
    pub span: Span,
}

impl Spanned for RecordComprehension {
    fn span(&self) -> source_span::Span {
        self.span
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
            span: open_brace_pipe.span.union(close_brace_pipe.span),
            open_brace_pipe,
            key_expression,
            expression,
            query,
            close_brace_pipe,
        })
    }
}
