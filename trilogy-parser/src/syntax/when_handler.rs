use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct WhenHandler {
    pub when: Token,
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub strategy: HandlerStrategy,
    pub span: Span,
}

impl WhenHandler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let when = parser
            .expect(KwWhen)
            .expect("Caller should have found this");
        let pattern = Pattern::parse(parser)?;
        let guard = parser
            .expect(KwIf)
            .ok()
            .map(|_| Expression::parse(parser))
            .transpose()?;
        let strategy = HandlerStrategy::parse(parser)?;

        Ok(Self {
            span: when.span.union(strategy.span()),
            when,
            guard,
            pattern,
            strategy,
        })
    }
}

impl Spanned for WhenHandler {
    fn span(&self) -> Span {
        self.span
    }
}
