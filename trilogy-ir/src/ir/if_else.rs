use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct IfElse {
    span: Span,
    pub condition: Expression,
    pub when_true: Expression,
    pub when_false: Expression,
}
