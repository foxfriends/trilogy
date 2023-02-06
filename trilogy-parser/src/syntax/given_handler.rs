use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct GivenHandler {
    start: Token,
    pub head: RuleHead,
    pub body: Option<Query>,
}

impl Spanned for GivenHandler {
    fn span(&self) -> Span {
        match &self.body {
            None => self.start.span.union(self.head.span()),
            Some(body) => self.start.span.union(body.span()),
        }
    }
}
