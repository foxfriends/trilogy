use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    start: Token,
    pub head: FunctionHead,
    pub body: Expression,
}
