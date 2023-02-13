use super::*;
use crate::{Parser, Spanned};
use source_span::Span;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct BindingPattern {
    pub mutable: MutModifier,
    pub identifier: Identifier,
}

impl BindingPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let mutable = MutModifier::parse(parser);
        let identifier = Identifier::parse(parser)?;
        Ok(Self {
            mutable,
            identifier,
        })
    }

    pub(crate) fn is_immutable(&self) -> bool {
        matches!(self.mutable, MutModifier::Not)
    }
}

impl Spanned for BindingPattern {
    fn span(&self) -> Span {
        match &self.mutable {
            MutModifier::Not => self.identifier.span(),
            MutModifier::Mut(token) => token.span.union(self.identifier.span()),
        }
    }
}
