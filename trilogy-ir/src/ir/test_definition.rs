use super::*;
use crate::Converter;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct TestDefinition {
    pub name: String,
    pub body: Expression,
}

impl TestDefinition {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::TestDefinition) -> Self {
        Self {
            name: ast.name.value(),
            body: Expression::convert_block(converter, ast.body),
        }
    }
}
