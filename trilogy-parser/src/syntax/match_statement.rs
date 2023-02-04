use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct MatchStatement {
    start: Token,
    pub expression: Expression,
    pub cases: Vec<MatchStatementCase>,
}

#[derive(Clone, Debug)]
pub struct MatchStatementCase {
    start: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Expression>,
    pub body: Block,
}
