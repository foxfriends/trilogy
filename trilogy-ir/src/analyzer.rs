use crate::ir::Module;
use crate::scope::Scope;
use crate::{Error, Id};
use trilogy_parser::syntax;

pub trait Resolver {
    fn resolve(&self, locator: &str) -> String;
}

pub struct Analyzer<'a> {
    resolver: &'a dyn Resolver,
    errors: Vec<Error>,
    scope: Scope,
}

impl<'a> Analyzer<'a> {
    pub fn new(resolver: &'a dyn Resolver) -> Self {
        Self {
            resolver,
            errors: vec![],
            scope: Scope::default(),
        }
    }

    pub fn analyze(&mut self, document: syntax::Document) -> Module {
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

    pub(crate) fn pop_scope(&mut self) {
        self.scope.pop();
    }

    pub(crate) fn declare(&mut self, name: String) -> Id {
        self.scope.declare(name)
    }

    pub(crate) fn temporary(&mut self) -> Id {
        self.scope.invent()
    }

    pub(crate) fn declared(&mut self, name: &str) -> Option<&Id> {
        self.scope.declared(name)
    }

    pub(crate) fn resolve(&self, locator: &str) -> String {
        self.resolver.resolve(locator)
    }
}
