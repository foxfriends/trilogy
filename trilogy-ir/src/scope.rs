use crate::symbol::{Id, SymbolTable};

#[derive(Default, Debug)]
pub(crate) struct Scope {
    parent: Option<Box<Scope>>,
    symbols: SymbolTable,
}

impl Scope {
    pub fn push(&mut self) {
        let parent = std::mem::take(self);
        self.parent = Some(Box::new(parent));
    }

    pub fn pop(&mut self) {
        *self = *self.parent.take().unwrap();
    }

    pub fn declare(&mut self, name: String) -> Id {
        self.symbols.reusable(name)
    }

    pub fn declared(&mut self, name: &str) -> Option<&Id> {
        self.symbols.reuse(name)
    }
}
