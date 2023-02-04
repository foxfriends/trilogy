use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct RuleDefinition {
    start: Token,
    pub head: RuleHead,
    pub body: Option<Query>,
}
