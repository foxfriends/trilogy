use source_span::Span;
use super::*;

#[derive(Clone, Debug)]
pub struct ArrayComprehension {
    span: Span,
    pub expression: Expression,
    pub query: Query,
}
