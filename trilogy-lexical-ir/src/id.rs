use super::ItemKey;
use source_span::Span;
use std::hash::{self, Hash};
use std::sync::Arc;
use trilogy_parser::syntax::Identifier;
use trilogy_parser::Spanned;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Definition {
    pub span: Span,
    pub name: String,
    pub is_mutable: bool,
}

#[derive(Clone, Debug)]
pub enum Id {
    Id(Arc<Definition>),
    Item(ItemKey),
    Closure(Arc<Span>),
}

impl Id {
    pub fn new_immutable(identifier: &Identifier) -> Self {
        Self::Id(Arc::new(Definition {
            span: identifier.span(),
            name: identifier.as_ref().to_owned(),
            is_mutable: false,
        }))
    }

    pub fn new_mutable(identifier: &Identifier) -> Self {
        Self::Id(Arc::new(Definition {
            span: identifier.span(),
            name: identifier.as_ref().to_owned(),
            is_mutable: true,
        }))
    }

    pub fn new_item(key: ItemKey) -> Self {
        Self::Item(key)
    }

    pub fn new_closure(span: Span) -> Self {
        Self::Closure(Arc::new(span))
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Id(def) => Some(&def.name),
            Self::Item(key) => Some(&key.name),
            _ => None,
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Id::Id(ident) => ident.is_mutable,
            _ => false,
        }
    }
}

impl Eq for Id {}
impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (Id::Id(lhs), Id::Id(rhs)) => Arc::ptr_eq(lhs, rhs),
            (Id::Item(lhs), Id::Item(rhs)) => lhs == rhs,
            (Id::Closure(lhs), Id::Closure(rhs)) => Arc::ptr_eq(lhs, rhs),
            _ => false,
        }
    }
}

impl Hash for Id {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Id::Id(def) => Arc::as_ptr(def).hash(state),
            Id::Item(item) => item.hash(state),
            Id::Closure(span) => Arc::as_ptr(span).hash(state),
        }
    }
}
