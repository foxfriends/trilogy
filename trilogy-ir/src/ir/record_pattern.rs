use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct RecordPattern {
    span: Span,
    pub elements: Vec<Pattern>,
    pub rest: Option<Pattern>,
}
