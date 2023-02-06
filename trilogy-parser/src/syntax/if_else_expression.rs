use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct IfElseExpression {
    start: Token,
    pub condition: Expression,
    pub when_true: Expression,
    pub when_false: Expression,
}
