use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct SetComprehension {
    pub open_bracket_pipe: Token,
    pub expression: Expression,
    pub query: Query,
    pub close_bracket_pipe: Token,
    pub span: Span,
}

impl Spanned for SetComprehension {
    fn span(&self) -> source_span::Span {
        self.span
    }
}

impl SetComprehension {
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        open_bracket_pipe: Token,
        expression: Expression,
    ) -> SyntaxResult<Self> {
        let query = Query::parse(parser)?;
        let close_bracket_pipe = parser
            .expect(CBrackPipe)
            .map_err(|token| parser.expected(token, "expected `|]` to end set comprehension"))?;
        Ok(Self {
            span: open_bracket_pipe.span.union(close_bracket_pipe.span),
            open_bracket_pipe,
            expression,
            query,
            close_bracket_pipe,
        })
    }
}
