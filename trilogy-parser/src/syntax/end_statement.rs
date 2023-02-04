use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct EndStatement {
    start: Token,
    pub expression: Option<Expression>,
}
