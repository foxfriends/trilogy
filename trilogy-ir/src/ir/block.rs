use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Block {
    span: Span,
    pub statements: Vec<Statement>,
}
