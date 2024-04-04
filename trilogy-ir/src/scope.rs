use crate::symbol::{Id, Symbol, SymbolTable};
use source_span::Span;

#[derive(Default, Debug)]
pub(crate) struct Scope {
    parent: Option<Box<Scope>>,
    symbols: SymbolTable,
    // A pseudoscope is used for `not` queries, where shadowing is not possible, but
    // new bindings are also not part of the parent scope
    pseudo: bool,
}

impl Scope {
    pub fn push(&mut self) {
        let parent = std::mem::take(self);
        self.parent = Some(Box::new(parent));
    }

    pub fn push_pseudo(&mut self) {
        let parent = std::mem::take(self);
        self.parent = Some(Box::new(parent));
        self.pseudo = true;
    }

    pub fn pop(&mut self) {
        *self = *self.parent.take().unwrap();
    }

    pub fn declare(&mut self, name: String, is_mutable: bool, span: Span) -> Symbol {
        if self.pseudo {
            if let Some(declared) = self.declared_pseudo(&name) {
                return declared.clone();
            }
        }
        self.symbols.reusable(name, is_mutable, span).clone()
    }

    pub fn invent(&mut self) -> Id {
        self.symbols.invent()
    }

    pub fn declared(&self, name: &str) -> Option<&Symbol> {
        self.symbols
            .reuse(name)
            .or_else(|| self.parent.as_ref()?.declared(name))
    }

    fn declared_pseudo(&self, name: &str) -> Option<&Symbol> {
        self.symbols.reuse(name).or_else(|| {
            if self.pseudo {
                self.parent.as_ref()?.declared_pseudo(name)
            } else {
                None
            }
        })
    }

    pub fn declared_no_shadow(&self, name: &str) -> Option<&Symbol> {
        self.symbols.reuse(name)
    }
}
