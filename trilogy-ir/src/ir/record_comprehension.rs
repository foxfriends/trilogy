use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct RecordComprehension {
    span: Span,
    pub expression: Expression,
    pub query: Query,
}
