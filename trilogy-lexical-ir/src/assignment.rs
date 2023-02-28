use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Assignment {
    span: Span,
    lvalue: LValue,
    rvalue: Evaluation,
}
