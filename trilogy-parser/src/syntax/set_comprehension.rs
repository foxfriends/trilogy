use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct SetComprehension {
    start: Token,
    pub expression: Expression,
    pub query: Query,
    end: Token,
}
