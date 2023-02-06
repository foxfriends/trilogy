use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct BreakExpression {
    start: Token,
    pub expression: Expression,
}
