use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Iterator {
    span: Span,
    pub value: Expression,
    pub query: Expression,
}
