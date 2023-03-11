use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct LValue {
    pub span: Span,
    pub container: Evaluation,
    pub property: Evaluation,
}
