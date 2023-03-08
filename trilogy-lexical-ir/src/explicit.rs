use super::*;
use source_span::Span;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ExplicitStrategy {
    Equal,
    In,
}

#[derive(Clone, Debug)]
pub struct Explicit {
    span: Span,
    lhs: Evaluation,
    rhs: Evaluation,
    strategy: ExplicitStrategy,
}
