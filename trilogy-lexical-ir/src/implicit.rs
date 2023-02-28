use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Implicit {
    span: Span,
    strategy: Evaluation,
    parameters: Vec<Evaluation>,
}
