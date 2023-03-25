use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Module {
    span: Span,
    pub parameters: Vec<Pattern>,
    pub definitions: Vec<Definition>,
}
