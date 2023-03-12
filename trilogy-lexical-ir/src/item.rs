use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Item {
    pub span: Span,
    pub source: Code,
}
