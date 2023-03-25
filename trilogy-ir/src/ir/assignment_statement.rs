use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct AssignmentStatement {
    span: Span,
    pub lhs: Expression,
    pub rhs: Expression,
}
