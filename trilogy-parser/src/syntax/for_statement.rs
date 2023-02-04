use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ForStatement {
    pub branches: Vec<ForStatementBranch>,
    pub else_block: Option<Block>,
}

#[derive(Clone, Debug)]
pub struct ForStatementBranch {
    start: Token,
    pub query: Query,
    pub body: Block,
}
