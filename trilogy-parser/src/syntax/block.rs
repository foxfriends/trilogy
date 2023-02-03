use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct Block {
    start: Token,
    pub statements: Vec<Statement>,
    end: Token,
}
