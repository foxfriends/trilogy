use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub parameters: Vec<Identifier>,
    definitions: Definitions,
}

impl Module {
    pub(crate) fn convert(converter: &mut Converter, ast: syntax::Document) -> Self {
        let span = ast.span();
        let definitions = Definitions::convert(converter, ast.definitions);
        Self {
            span,
            parameters: vec![],
            definitions,
        }
    }

    pub(crate) fn convert_module(converter: &mut Converter, ast: syntax::TypeDefinition) -> Self {
        converter.push_scope();
        let span = ast.span();
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
        }
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions.0
    }

    pub fn definitions_mut(&mut self) -> &mut [Definition] {
        &mut self.definitions.0
    }
}
