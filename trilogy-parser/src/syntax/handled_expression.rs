use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct HandledExpression {
    start: Token,
    pub expression: Expression,
    pub handlers: Vec<Handler>,
}

impl Spanned for HandledExpression {
    fn span(&self) -> Span {
        self.start.span.union(if self.handlers.is_empty() {
            self.expression.span()
        } else {
            self.handlers.span()
        })
    }
}

impl HandledExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwWith)
            .map_err(|token| parser.expected(token, "expected `with`"))?;

        let expression = Expression::parse(parser)?;

        let mut handlers = vec![];
        loop {
            if let Err(token) = parser.check([KwGiven, KwWhen, KwElse]) {
                let error = SyntaxError::new(
                    token.span,
                    "expected `when`, `given`, or `else` to start an effect handler",
                );
                parser.error(error.clone());
                return Err(error);
            }
            let handler = Handler::parse(parser)?;
            let end = matches!(handler, Handler::Else(..));
            handlers.push(handler);
            if end {
                return Ok(Self {
                    start,
                    expression,
                    handlers,
                });
            }
        }
    }
}
