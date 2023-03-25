use super::*;
use crate::Analyzer;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct TestDefinition {
    pub name: StringLiteral,
    pub body: Expression,
}

impl TestDefinition {
    pub(super) fn convert(_analyzer: &mut Analyzer, _ast: syntax::TestDefinition) -> Self {
        todo!()
    }
}
