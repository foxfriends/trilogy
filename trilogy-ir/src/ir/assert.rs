use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Assert {
    span: Span,
    pub message: Expression,
    pub assertion: Expression,
}
