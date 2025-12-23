use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct SlotDefinition {
    pub name: Identifier,
    pub is_mutable: bool,
    pub value: Expression,
}

impl SlotDefinition {
    pub(super) fn declare(name: Identifier, is_mutable: bool) -> Self {
        Self {
            // This value is a temporary, and will be replaced, so it's not a big deal
            // that it's `unit` right now.
            value: Expression::unit(name.span),
            is_mutable,
            name,
        }
    }

    pub fn span(&self) -> Span {
        self.name.span.union(self.value.span)
    }
}
