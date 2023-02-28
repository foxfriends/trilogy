use super::*;
use source_span::Span;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ExplicitStrategy {
    In,
    Equal,
}

#[derive(Clone, Debug)]
pub struct Explicit {
    span: Span,
    lhs: Evaluation,
    rhs: Evaluation,
    strategy: ExplicitStrategy,
}
