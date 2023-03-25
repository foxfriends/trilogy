use source_span::Span;

use super::*;

#[derive(Clone, Debug)]
pub struct Handler {
    span: Span,
    pub pattern: Pattern,
    pub guard: Expression,
    pub body: Expression,
}
