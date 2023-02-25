use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Rename {
    pub span: Span,
    pub item: Scope,
}
