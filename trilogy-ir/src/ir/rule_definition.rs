use super::*;

#[derive(Clone, Debug)]
pub struct RuleDefinition {
    pub name: Identifier,
    pub overloads: Vec<Rule>,
}

impl RuleDefinition {
    pub(super) fn declare(name: Identifier) -> Self {
        Self {
            name,
            overloads: vec![],
        }
    }
}
