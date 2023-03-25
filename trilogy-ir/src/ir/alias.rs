use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Alias {
    span: Span,
    pub value: Expression,
}
