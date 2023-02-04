use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct YieldStatement {
    start: Token,
    pub expression: Expression,
}
