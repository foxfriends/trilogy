use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct ContinueExpression {
    start: Token,
    pub expression: Expression,
}
