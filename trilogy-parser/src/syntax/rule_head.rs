use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct RuleHead {
    pub name: Identifier,
    pub parameters: Vec<Pattern>,
    end: Token,
}
