use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub enum LValue {
    Member {
        span: Span,
        container: Evaluation,
        property: Evaluation,
    },
    Rebind(Reference),
}
