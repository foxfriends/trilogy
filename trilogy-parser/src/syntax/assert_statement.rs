use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct AssertStatement {
    start: Token,
    pub message: Option<Expression>,
    pub assertion: Expression,
}
