use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ReturnStatement {
    start: Token,
    pub expression: Option<Expression>,
}
