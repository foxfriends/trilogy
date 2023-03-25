use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Pack {
    span: Span,
    pub values: Vec<Element>,
}

#[derive(Clone, Debug)]
pub struct Element {
    span: Span,
    pub expression: Expression,
    pub is_spread: bool,
}
