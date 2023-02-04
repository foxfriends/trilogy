use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct IfStatement {
    pub branches: Vec<IfBranch>,
    pub if_false: Option<Block>,
}

#[derive(Clone, Debug)]
pub struct IfBranch {
    start: Token,
    pub condition: Expression,
    pub body: Block,
}
