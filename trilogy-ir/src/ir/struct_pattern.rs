use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct StructPattern {
    span: Span,
    pub atom: String,
    pub pattern: Pattern,
}
