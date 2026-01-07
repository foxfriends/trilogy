use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub parameters: Vec<Identifier>,
    definitions: Definitions,
    pub definitions_span: Span,
}

impl Module {
    pub(crate) fn convert(converter: &mut Converter, ast: syntax::Document) -> Self {
        let span = ast.span();
        let definitions = Definitions::convert(converter, ast.definitions);
        Self {
            span,
            parameters: vec![],
            definitions,
            definitions_span: span,
        }
    }

    pub(crate) fn convert_module(converter: &mut Converter, ast: syntax::TypeDefinition) -> Self {
        converter.push_scope();
        let span = ast.span();
        let definitions_span = ast.open_brace.span.union(ast.close_brace.span);
        let parameters: Vec<_> = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Identifier::declare(converter, param))
            .collect();
        let definitions = Definitions::convert(converter, ast.definitions);
        converter.pop_scope();
        Self {
            span,
            parameters,
            definitions,
            definitions_span,
        }
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions.0
    }

    pub fn definitions_mut(&mut self) -> &mut [Definition] {
        &mut self.definitions.0
    }
}
