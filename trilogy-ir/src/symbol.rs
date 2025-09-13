use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use source_span::Span;

#[derive(Clone)]
pub struct Id {
    pub declaration_span: Span,
    pub is_mutable: bool,
    tag: Arc<String>,
}

impl Id {
    pub fn name(&self) -> &str {
        &self.tag
    }
}

impl Eq for Id {}
impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.tag, &other.tag)
    }
}

impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.tag).hash(state)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_mutable {
            write!(f, "[{}] let mut {}", self.declaration_span, self.tag)
        } else {
            write!(f, "[{}] let {}", self.declaration_span, self.tag)
        }
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tag)
    }
}

#[derive(Default, Debug)]
pub(crate) struct SymbolTable {
    symbols: HashMap<String, Id>,
}

impl SymbolTable {
    pub fn invent(&mut self, declaration_span: Span) -> Id {
        Id {
            declaration_span,
            is_mutable: false,
            tag: Arc::new(String::from("<intermediate value>")),
        }
    }

    pub fn reusable(&mut self, tag: String, is_mutable: bool, declaration_span: Span) -> &Id {
        self.symbols.entry(tag.clone()).or_insert_with(|| Id {
            declaration_span,
            is_mutable,
            tag: Arc::new(tag),
        })
    }

    pub fn reuse(&self, tag: &str) -> Option<&Id> {
        self.symbols.get(tag)
    }
}
