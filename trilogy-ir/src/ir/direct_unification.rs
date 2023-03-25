use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct DirectUnification {
    span: Span,
    pub pattern: Pattern,
    pub expression: Expression,
}
