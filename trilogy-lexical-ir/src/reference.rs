use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Reference {
    pub span: Span,
    pub id: Id,
}
