use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Application {
    span: Span,
    pub function: Expression,
    pub argument: Expression,
}
