use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct ParenthesizedQuery {
    start: Token,
    pub query: Query,
    end: Token,
}
