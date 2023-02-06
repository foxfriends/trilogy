use super::*;
use crate::Spanned;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct BindingPattern {
    pub mutable: MutModifier,
    pub identifier: Identifier,
}

impl Spanned for BindingPattern {
    fn span(&self) -> Span {
        match &self.mutable {
            MutModifier::Not => self.identifier.span(),
            MutModifier::Mut(token) => token.span.union(self.identifier.span()),
        }
    }
}
