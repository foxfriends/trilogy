use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Reference {
    pub span: Span,
    pub target: Id,
}

impl Reference {
    pub fn new(span: Span, target: Id) -> Self {
        Self { span, target }
    }

    pub fn temp(span: Span) -> Self {
        Self {
            span,
            target: Id::new_temporary(span),
        }
    }
}
