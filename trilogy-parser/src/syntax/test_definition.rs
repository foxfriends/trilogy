use super::*;

#[derive(Clone, Debug)]
pub struct TestDefinition {
    pub name: StringLiteral,
    pub body: Block,
}
