use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct BreakExpression {
    start: Token,
    pub expression: Expression,
}
