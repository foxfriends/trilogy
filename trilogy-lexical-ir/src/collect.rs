use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Collect {
    span: Span,
    strategy: CollectStrategy,
    query: Vec<Code>,
    body: Vec<Code>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CollectStrategy {
    Scalar,
    Array,
    Record,
    Set,
    List,
    Sequence,
}
