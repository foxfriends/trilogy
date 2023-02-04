use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct NegativePattern {
    start: Token,
    pub pattern: ValuePattern,
}
