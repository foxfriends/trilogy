use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct SetPattern {
    start: Token,
    pub elements: Vec<Pattern>,
    pub rest: Option<Pattern>,
    end: Token,
}

impl Spanned for SetPattern {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}
