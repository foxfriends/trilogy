use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct EndExpression {
    start: Token,
    pub expression: Option<Expression>,
}
