use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub operand: Expression,
}

#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Negate(Token),
    Not(Token),
    Invert(Token),
    Yield(Token),
}
