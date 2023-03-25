use source_span::Span;
use super::*;

#[derive(Clone, Debug)]
pub struct StructPattern {
    span: Span,
    pub atom: AtomLiteral,
    pub pattern: Pattern,
}
