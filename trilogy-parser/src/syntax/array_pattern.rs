use super::*;
use crate::spanned::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ArrayPattern {
    start: Token,
    pub head: Option<Pattern>,
    pub body: Vec<Pattern>,
    pub tail: Option<Pattern>,
    end: Token,
}

impl Spanned for ArrayPattern {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}
