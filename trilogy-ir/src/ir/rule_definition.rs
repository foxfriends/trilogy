use super::*;
use source_span::Span;

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

    pub fn span(&self) -> Span {
        self.overloads
            .first()
            .unwrap()
            .span
            .union(self.overloads.last().unwrap().span)
    }
}
