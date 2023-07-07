use std::sync::Arc;

use crate::ir::{Module, ModuleCell};
use crate::scope::Scope;
use crate::{Error, Id, Resolver};
use trilogy_parser::syntax;

pub struct Analyzer<'a> {
    errors: Vec<Error>,
    resolver: Box<dyn Resolver + 'a>,
    scope: Scope,
}

impl<'a> Analyzer<'a> {
    pub fn new<R: Resolver + 'a>(resolver: R) -> Self {
        Self {
            errors: vec![],
            resolver: Box::new(resolver),
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

    pub(crate) fn resolve(&mut self, path: &str) -> Arc<ModuleCell> {
        self.resolver.resolve(path)
    }

    pub(crate) fn location(&self) -> String {
        self.resolver.location()
    }
}
