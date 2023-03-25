use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct SetComprehension {
    span: Span,
    pub expression: Expression,
    pub query: Query,
}
