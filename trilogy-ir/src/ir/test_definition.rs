use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct TestDefinition {
    span: Span,
    pub name: StringLiteral,
    pub body: Expression,
}
