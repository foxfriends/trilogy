use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct SetPattern {
    start: Token,
    pub elements: Vec<Pattern>,
    pub rest: Option<Pattern>,
    end: Token,
}
