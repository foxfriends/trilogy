use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct RecordPattern {
    start: Token,
    pub elements: Vec<(Pattern, Pattern)>,
    pub rest: Option<Pattern>,
    end: Token,
}
