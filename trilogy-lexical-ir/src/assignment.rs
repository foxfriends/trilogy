use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Assignment {
    pub span: Span,
    pub lvalue: LValue,
    pub rvalue: Evaluation,
}

impl Assignment {
    pub fn new(span: Span, lvalue: LValue, rvalue: Evaluation) -> Self {
        Self {
            span,
            lvalue,
            rvalue,
        }
    }
}
