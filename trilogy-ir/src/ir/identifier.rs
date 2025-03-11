use crate::{Converter, Id};
use source_span::Span;
use std::fmt::Display;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Identifier {
    pub span: Span,
    pub declaration_span: Span,
    pub is_mutable: bool,
    pub id: Id,
}

impl Identifier {
    pub(super) fn temporary(converter: &mut Converter, span: Span) -> Self {
        let id = converter.temporary(span);
        Self {
            declaration_span: span,
            span,
            is_mutable: false,
            id,
        }
    }

    pub(crate) fn declare_binding(
        converter: &mut Converter,
        binding: syntax::BindingPattern,
    ) -> Identifier {
        let span = binding.span();
        let is_mutable = binding.is_mutable();
        let id @ Id {
            declaration_span, ..
        } = converter
            .declare(binding.identifier.into(), is_mutable, span)
            .clone();
        Self {
            span,
            declaration_span,
            is_mutable,
            id,
        }
    }

    pub(crate) fn declare(converter: &mut Converter, identifier: syntax::Identifier) -> Identifier {
        let span = identifier.span();
        let id @ Id {
            is_mutable,
            declaration_span,
            ..
        } = converter.declare(identifier.into(), false, span).clone();
        Self {
            span,
            declaration_span,
            is_mutable,
            id,
        }
    }

    pub(crate) fn unresolved(
        converter: &mut Converter,
        identifier: syntax::Identifier,
    ) -> Identifier {
        // Quietly this is the same thing, but we call it unresolved out there so it looks better.
        Self::declare(converter, identifier)
    }

    pub(crate) fn declared(
        converter: &mut Converter,
        identifier: &syntax::Identifier,
    ) -> Option<Identifier> {
        let span = identifier.span();
        let id @ Id {
            declaration_span,
            is_mutable,
            ..
        } = converter.declared(identifier.as_ref())?.clone();
        Some(Self {
            span,
            declaration_span,
            is_mutable,
            id,
        })
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}
