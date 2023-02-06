use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct ReturnExpression {
    start: Token,
    pub expression: Expression,
}
