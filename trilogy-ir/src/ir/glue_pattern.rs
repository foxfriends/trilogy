use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct GluePattern {
    span: Span,
    pub lhs: Pattern,
    pub rhs: Pattern,
}
