use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Evaluation {
    pub span: Span,
    pub value: Value,
}

impl Evaluation {
    pub fn new(span: Span, value: Value) -> Self {
        Self { span, value }
    }
}
