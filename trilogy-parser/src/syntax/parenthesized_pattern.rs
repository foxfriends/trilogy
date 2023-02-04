use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ParenthesizedPattern {
    start: Token,
    pub pattern: Pattern,
    end: Token,
}
