use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct LValue {
    span: Span,
    container: Evaluation,
    property: Evaluation,
}
