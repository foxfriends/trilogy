use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Evaluation {
    span: Span,
    operation: Operation,
}
