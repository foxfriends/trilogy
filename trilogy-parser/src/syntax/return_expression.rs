use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ReturnExpression {
    start: Token,
    pub expression: Expression,
}
