use super::*;
use crate::spanned::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct AssertStatement {
    start: Token,
    pub message: Option<Expression>,
    pub assertion: Expression,
}

impl Spanned for AssertStatement {
    fn span(&self) -> Span {
        self.start.span.union(self.assertion.span())
    }
}
