use super::*;
use crate::Analyzer;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct TestDefinition {
    pub name: String,
    pub body: Expression,
}

impl TestDefinition {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::TestDefinition) -> Self {
        Self {
            name: ast.name.value(),
            body: Expression::convert_block(analyzer, ast.body),
        }
    }
}
