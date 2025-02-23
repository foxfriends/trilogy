use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct TestDefinition {
    pub span: Span,
    pub name: String,
    pub body: Expression,
    pub negated: bool,
}

impl TestDefinition {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::TestDefinition) -> Self {
        Self {
            span: ast.test.span.union(ast.body.span()),
            name: ast.name.value(),
            body: Expression::convert_block(converter, ast.body),
            negated: ast.not.is_some(),
        }
    }
}
