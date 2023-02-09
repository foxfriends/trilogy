use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RecordPattern {
    start: Token,
    pub elements: Vec<(Pattern, Pattern)>,
    pub rest: Option<Pattern>,
    end: Token,
}

impl Spanned for RecordPattern {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}
