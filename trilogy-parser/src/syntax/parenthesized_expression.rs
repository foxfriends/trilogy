use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ParenthesizedExpression {
    start: Token,
    pub pattern: Expression,
    end: Token,
}
