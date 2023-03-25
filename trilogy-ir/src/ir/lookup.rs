use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Lookup {
    span: Span,
    pub path: Identifier,
    pub patterns: Vec<Pattern>,
}
