use crate::ir::Module;
use crate::scope::Scope;
use crate::{Error, Id};
use source_span::Span;
use trilogy_parser::syntax;

pub trait Resolver {
    fn resolve(&self, locator: &str) -> String;
}

pub struct Converter<'a> {
    resolver: &'a dyn Resolver,
    source: &'a str,
    errors: Vec<Error>,
    scope: Scope,
}

impl<'a> Converter<'a> {
    pub fn new(resolver: &'a dyn Resolver, source: &'a str) -> Self {
        Self {
            resolver,
            errors: vec![],
            scope: Scope::default(),
            source,
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

    pub(crate) fn declare(&mut self, name: String, is_mutable: bool, span: Span) -> Id {
        self.scope.declare(name, is_mutable, span)
    }

    pub(crate) fn temporary(&mut self, span: Span) -> Id {
        self.scope.invent(span)
    }

    pub(crate) fn declared(&mut self, name: &str) -> Option<&Id> {
        self.scope.declared(name)
    }

    pub(crate) fn declared_no_shadow(&mut self, name: &str) -> Option<&Id> {
        self.scope.declared_no_shadow(name)
    }

    pub(crate) fn resolve(&self, locator: &str) -> String {
        self.resolver.resolve(locator)
    }

    pub(crate) fn get_source(&self, span: Span) -> String {
        let mut lines = self
            .source
            .lines()
            .skip(span.start().line)
            .take(span.last().line - span.start().line + 1)
            .map(|line| line.to_owned())
            .collect::<Vec<_>>();
        lines[0] = lines[0]
            .chars()
            .skip(span.start().column)
            .collect::<String>();
        let last = lines.last_mut().unwrap();
        *last = last.chars().take(span.end().column).collect::<String>();
        lines.join("")
    }
}
