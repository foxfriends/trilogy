use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Collect {
    pub span: Span,
    pub strategy: CollectStrategy,
    pub body: Vec<Code>,
    pub direction: Direction,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CollectStrategy {
    Void,   // For loop
    Scalar, // Existence
    Array,  // Comprehension
    Record,
    Set,
}
