use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct ElementUnification {
    span: Span,
    pub pattern: Pattern,
    pub expression: Expression,
}
