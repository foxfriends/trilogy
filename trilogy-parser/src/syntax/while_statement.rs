use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct WhileStatement {
    start: Token,
    pub condition: Expression,
    pub body: Block,
}
