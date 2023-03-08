use source_span::Span;
use std::hash::{self, Hash};
use std::sync::Arc;

#[derive(Debug)]
pub struct Identifier {
    definition: Option<Span>,
    name: Option<String>,
    is_mutable: bool,
}

#[derive(Clone, Debug)]
pub struct Id(Arc<Identifier>);

impl Id {
    fn is_mutable(&self) -> bool {
        self.0.is_mutable
    }
}

impl Eq for Id {}
impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Id {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state)
    }
}
