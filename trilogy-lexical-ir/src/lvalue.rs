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

impl LValue {
    pub fn member(span: Span, container: Evaluation, property: Evaluation) -> Self {
        Self::Member {
            span,
            container,
            property,
        }
    }
}
