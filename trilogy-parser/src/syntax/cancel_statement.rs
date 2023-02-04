use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct CancelStatement {
    start: Token,
    pub expression: Option<Expression>,
}
