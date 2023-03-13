use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Direction {
    pub span: Span,
    pub body: Step,
}
