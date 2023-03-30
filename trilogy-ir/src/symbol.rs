use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Id(Arc<Option<String>>);

impl Id {
    fn new(tag: String) -> Self {
        Self(Arc::new(Some(tag)))
    }
}

impl Eq for Id {}
impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state)
    }
}

#[derive(Default, Debug)]
pub(crate) struct SymbolTable {
    symbols: HashMap<String, Id>,
}

impl SymbolTable {
    pub fn invent(&mut self) -> Id {
        Id::new(String::from("<intermediate value>"))
    }

    pub fn reusable(&mut self, tag: String) -> Id {
        self.symbols
            .entry(tag.clone())
            .or_insert_with(|| Id::new(tag))
            .clone()
    }

    pub fn reuse(&mut self, tag: &str) -> Option<&Id> {
        self.symbols.get(tag)
    }
}
