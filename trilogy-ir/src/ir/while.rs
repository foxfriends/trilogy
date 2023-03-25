use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct While {
    span: Span,
    pub condition: Expression,
    pub body: Expression,
}
