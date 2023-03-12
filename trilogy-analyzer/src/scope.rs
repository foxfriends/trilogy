use std::collections::HashMap;
use std::sync::Arc;
use trilogy_lexical_ir::Id;

#[derive(Clone, Default, Debug)]
pub(crate) struct Scope {
    parent: Option<Arc<Scope>>,
    bindings: HashMap<String, Id>,
}

impl Scope {
    pub(crate) fn extend(&mut self, bindings: HashMap<String, Id>) {
        *self = Self {
            parent: Some(Arc::new(std::mem::take(self))),
            bindings,
        };
    }
}
