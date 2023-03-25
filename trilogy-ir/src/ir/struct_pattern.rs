use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct StructPattern {
    span: Span,
    pub atom: AtomLiteral,
    pub pattern: Pattern,
}
