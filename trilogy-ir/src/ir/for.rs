use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct For {
    span: Span,
    pub iterator: Query,
    pub body: Expression, // TODO: is a for loop just a comprehension into nothing?
}
