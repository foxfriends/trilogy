use crate::symbol::{Id, Symbol, SymbolTable};
use source_span::Span;

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

    pub fn declare(&mut self, name: String, is_mutable: bool, span: Span) -> &Symbol {
        self.symbols.reusable(name, is_mutable, span)
    }

    pub fn invent(&mut self) -> Id {
        self.symbols.invent()
    }

    pub fn declared(&self, name: &str) -> Option<&Symbol> {
        self.symbols
            .reuse(name)
            .or_else(|| self.parent.as_ref()?.declared(name))
    }

    pub fn declared_no_shadow(&self, name: &str) -> Option<&Symbol> {
        self.symbols.reuse(name)
    }
}
