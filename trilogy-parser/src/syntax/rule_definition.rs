use super::*;

#[derive(Clone, Debug)]
pub struct RuleDefinition {
    pub head: RuleHead,
    pub body: Option<Query>,
}
