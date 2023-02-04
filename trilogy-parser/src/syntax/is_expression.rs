use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct IsExpression {
    start: Token,
    pub query: Query,
}
