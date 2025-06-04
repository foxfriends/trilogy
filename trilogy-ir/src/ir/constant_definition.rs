use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct ConstantDefinition {
    pub name: Identifier,
    pub value: Expression,
}

impl ConstantDefinition {
    pub(super) fn declare(name: Identifier) -> Self {
        Self {
            // This value is a temporary, and will be replaced, so it's not a big deal
            // that it's `unit` right now.
            value: Expression::unit(name.span),
            name,
        }
    }

    pub fn span(&self) -> Span {
        self.name.span.union(self.value.span)
    }
}
