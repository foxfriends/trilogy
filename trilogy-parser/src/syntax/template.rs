use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct Template {
    start: Token,
    pub segments: Vec<TemplateSegment>,
    pub tag: Option<Identifier>,
}

impl Spanned for Template {
    fn span(&self) -> Span {
        let mut span = self.start.span;
        if !self.segments.is_empty() {
            span = span.union(self.segments.span());
        }
        if let Some(tag) = &self.tag {
            span = span.union(tag.span());
        }
        span
    }
}

#[derive(Clone, Debug, Spanned)]
pub struct TemplateSegment {
    pub interpolation: Expression,
    end: Token,
}
