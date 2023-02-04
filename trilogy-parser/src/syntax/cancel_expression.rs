use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct CancelExpression {
    start: Token,
    pub expression: Expression,
}
