use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Let {
    span: Span,
    pub query: Query,
    pub body: Expression,
}
