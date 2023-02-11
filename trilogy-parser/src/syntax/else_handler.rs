use super::*;
use crate::spanned::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ElseHandler {
    start: Token,
    pub identifier: Option<Identifier>,
    pub strategy: HandlerStrategy,
    pub body: Option<HandlerBody>,
}

impl Spanned for ElseHandler {
    fn span(&self) -> Span {
        match &self.body {
            None => self.start.span.union(self.strategy.span()),
            Some(body) => self.start.span.union(body.span()),
        }
    }
}
