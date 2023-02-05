use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ArrayPattern {
    start: Token,
    pub head: Option<Pattern>,
    pub body: Vec<Pattern>,
    pub tail: Option<Pattern>,
    end: Token,
}