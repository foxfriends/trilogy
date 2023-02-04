use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ContinueExpression {
    start: Token,
    pub expression: Expression,
}
