use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct LetStatement {
    start: Token,
    pub unification: Query,
}
