use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Assignment {
    pub span: Span,
    pub lvalue: LValue,
    pub rvalue: Evaluation,
}
