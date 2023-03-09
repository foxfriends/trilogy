use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Collect {
    span: Span,
    strategy: CollectStrategy,
    body: Vec<Code>,
    direction: Direction,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CollectStrategy {
    Void,   // For loop
    Scalar, // Existence
    Array,  // Comprehension
    Record,
    Set,
}
