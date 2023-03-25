use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct ArrayPattern {
    span: Span,
    pub head: Vec<Pattern>,
    pub rest: Option<Pattern>,
    pub tail: Vec<Pattern>,
}
