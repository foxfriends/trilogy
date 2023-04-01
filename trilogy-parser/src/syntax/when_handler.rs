use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct WhenHandler {
    start: Token,
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub strategy: HandlerStrategy,
    pub body: Option<HandlerBody>,
}

impl WhenHandler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwWhen)
            .expect("Caller should have found this");
        let pattern = Pattern::parse(parser)?;
        let guard = parser
            .expect(KwIf)
            .ok()
            .map(|_| Expression::parse(parser))
            .transpose()?;
        let strategy = HandlerStrategy::parse(parser)?;
        let body = if !matches!(strategy, HandlerStrategy::Yield(..)) {
            Some(HandlerBody::parse(parser)?)
        } else {
            None
        };

        Ok(Self {
            start,
            guard,
            pattern,
            strategy,
            body,
        })
    }
}

impl Spanned for WhenHandler {
    fn span(&self) -> Span {
        match &self.body {
            None => self.start.span.union(self.strategy.span()),
            Some(body) => self.start.span.union(body.span()),
        }
    }
}
