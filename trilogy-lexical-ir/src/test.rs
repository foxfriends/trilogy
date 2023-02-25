use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Test {
    span: Span,
    code: Code,
}
