use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct IteratorComprehension {
    start: Token,
    pub expression: Expression,
    pub query: Query,
    end: Token,
}