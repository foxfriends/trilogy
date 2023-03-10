use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Scope {
    pub span: Span,
    pub code: Vec<Code>,
}
