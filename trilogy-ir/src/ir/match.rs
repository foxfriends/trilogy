use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Match {
    span: Span,
    pub expression: Expression,
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug)]
pub struct Case {
    span: Span,
    pub pattern: Pattern,
    pub guard: Expression,
}
