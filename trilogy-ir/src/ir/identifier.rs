use crate::{symbol::Symbol, Analyzer, Id};
use source_span::Span;
use std::fmt::Display;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Identifier {
    pub span: Span,
    pub declaration_span: Span,
    pub is_mutable: bool,
    pub id: Id,
}

impl Identifier {
    pub(super) fn temporary(analyzer: &mut Analyzer, span: Span) -> Self {
        let id = analyzer.temporary();
        Self {
            declaration_span: span,
            span,
            is_mutable: false,
            id,
        }
    }

    pub(crate) fn declare_binding(
        analyzer: &mut Analyzer,
        binding: syntax::BindingPattern,
    ) -> Identifier {
        let span = binding.span();
        let is_mutable = binding.is_mutable();
        let Symbol {
            id,
            declaration_span,
            ..
        } = analyzer
            .declare(binding.identifier.into(), is_mutable, span)
            .clone();
        Self {
            span,
            declaration_span,
            is_mutable,
            id,
        }
    }

    pub(crate) fn declare(analyzer: &mut Analyzer, identifier: syntax::Identifier) -> Identifier {
        let span = identifier.span();
        let Symbol {
            id,
            is_mutable,
            declaration_span,
        } = analyzer.declare(identifier.into(), false, span).clone();
        Self {
            span,
            declaration_span,
            is_mutable,
            id,
        }
    }

    pub(crate) fn unresolved(
        analyzer: &mut Analyzer,
        identifier: syntax::Identifier,
    ) -> Identifier {
        // Quietly this is the same thing, but we call it unresolved out there so it looks better.
        Self::declare(analyzer, identifier)
    }

    pub(crate) fn declared(
        analyzer: &mut Analyzer,
        identifier: &syntax::Identifier,
    ) -> Option<Identifier> {
        let span = identifier.span();
        let Symbol {
            id,
            declaration_span,
            is_mutable,
            ..
        } = analyzer.declared(identifier.as_ref())?.clone();
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
