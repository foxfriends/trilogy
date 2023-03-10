use super::ItemKey;
use source_span::Span;
use std::hash::{self, Hash};
use std::sync::Arc;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Identifier {
    pub definition: Option<Span>,
    pub name: Option<String>,
    pub is_mutable: bool,
}

#[derive(Clone, Debug)]
pub enum Id {
    Id(Arc<Identifier>),
    Item(ItemKey),
}

impl Id {
    fn is_mutable(&self) -> bool {
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
            _ => false,
        }
    }
}

impl Hash for Id {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Id::Id(ident) => Arc::as_ptr(ident).hash(state),
            Id::Item(item) => item.hash(state),
        }
    }
}
