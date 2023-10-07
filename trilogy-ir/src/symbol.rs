use std::collections::HashMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use source_span::Span;

#[derive(Clone, Debug)]
pub struct Id(Arc<Option<String>>);

impl Id {
    fn new(tag: String) -> Self {
        Self(Arc::new(Some(tag)))
    }

    pub fn name(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub fn symbol(&self) -> String {
        match self.0.as_ref() {
            Some(s) => format!("{s}#{:x}", Arc::as_ptr(&self.0) as usize),
            None => format!("#{:x}", Arc::as_ptr(&self.0) as usize),
        }
    }

    pub fn as_ptr(&self) -> *const Option<String> {
        Arc::as_ptr(&self.0)
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

#[derive(Clone, Debug)]
pub(crate) struct Symbol {
    #[allow(dead_code)]
    pub declaration_span: Span,
    pub id: Id,
    pub is_mutable: bool,
}

#[derive(Default, Debug)]
pub(crate) struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn invent(&mut self) -> Id {
        Id::new(String::from("<intermediate value>"))
    }

    pub fn reusable(&mut self, tag: String, is_mutable: bool, span: Span) -> &Symbol {
        self.symbols.entry(tag.clone()).or_insert_with(|| Symbol {
            declaration_span: span,
            is_mutable,
            id: Id::new(tag),
        })
    }

    pub fn reuse(&self, tag: &str) -> Option<&Symbol> {
        self.symbols.get(tag)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_ref() {
            Some(name) => name.fmt(f),
            None => "<intermediate value>".fmt(f),
        }
    }
}
