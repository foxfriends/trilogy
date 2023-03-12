use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Test {
    pub span: Span,
    pub name: String,
    pub code: Code,
}
