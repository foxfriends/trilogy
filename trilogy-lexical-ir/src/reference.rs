use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Reference {
    span: Span,
    id: Id,
}
