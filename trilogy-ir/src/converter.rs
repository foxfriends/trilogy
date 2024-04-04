use crate::ir::Module;
use crate::scope::Scope;
use crate::symbol::Symbol;
use crate::{Error, Id};
use source_span::Span;
use trilogy_parser::syntax;

pub trait Resolver {
    fn resolve(&self, locator: &str) -> String;
}

pub struct Converter<'a> {
    resolver: &'a dyn Resolver,
    errors: Vec<Error>,
    scope: Scope,
}

impl<'a> Converter<'a> {
    pub fn new(resolver: &'a dyn Resolver) -> Self {
        Self {
            resolver,
            errors: vec![],
            scope: Scope::default(),
        }
    }

    pub fn convert(&mut self, document: syntax::Document) -> Module {
        Module::convert(self, document)
    }

    pub(crate) fn error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn errors(self) -> Vec<Error> {
        self.errors
    }

    pub(crate) fn push_scope(&mut self) {
        self.scope.push();
    }

    pub(crate) fn push_pseudo_scope(&mut self) {
        self.scope.push_pseudo();
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scope.pop();
    }

    pub(crate) fn declare(&mut self, name: String, is_mutable: bool, span: Span) -> Symbol {
        self.scope.declare(name, is_mutable, span)
    }

    pub(crate) fn temporary(&mut self) -> Id {
        self.scope.invent()
    }

    pub(crate) fn declared(&mut self, name: &str) -> Option<&Symbol> {
        self.scope.declared(name)
    }

    pub(crate) fn declared_no_shadow(&mut self, name: &str) -> Option<&Symbol> {
        self.scope.declared_no_shadow(name)
    }

    pub(crate) fn resolve(&self, locator: &str) -> String {
        self.resolver.resolve(locator)
    }
}
