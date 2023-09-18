use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub parameters: Vec<Expression>,
    definitions: Definitions,
}

impl Module {
    pub(crate) fn convert(analyzer: &mut Analyzer, ast: syntax::Document) -> Self {
        let span = ast.span();
        let definitions = Definitions::convert(analyzer, ast.definitions);
        Self {
            span,
            parameters: vec![],
            definitions,
        }
    }

    pub(crate) fn convert_module(analyzer: &mut Analyzer, ast: syntax::ModuleDefinition) -> Self {
        analyzer.push_scope();
        let span = ast.span();
        let parameters: Vec<_> = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Expression::reference(param.span(), Identifier::declare(analyzer, param)))
            .collect();
        let definitions = Definitions::convert(analyzer, ast.definitions);
        analyzer.pop_scope();
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
