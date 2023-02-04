use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct TestDefinition {
    start: Token,
    pub name: StringLiteral,
    pub body: Block,
}
