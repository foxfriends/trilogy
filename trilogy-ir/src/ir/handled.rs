use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Handled {
    span: Span,
    pub expression: Expression,
    pub handlers: Vec<Handler>,
}
